import { ModelBuilderError } from "./error.js";
import {
    BoolVar,
    BooleanExpression,
    NumericExpression,
    NumericVar,
    evaluateExpression,
    expressionOwner,
    type ExpressionValue,
    type VariableReference,
} from "./expressions.js";

const INTERNAL_PREFIX = "$__rooc_ts_";

type RawTaggedValue =
    | { readonly type: "Bool"; readonly value: boolean }
    | { readonly type: "Int"; readonly value: number }
    | { readonly type: "Real"; readonly value: number };

type RawAssignment = {
    readonly name: string;
    readonly value: number | RawTaggedValue;
};

type RawSolution = {
    readonly assignment: readonly RawAssignment[];
    readonly constraints: Map<string, number> | Record<string, number>;
    readonly value: number;
};

/** A recursive selection accepted by FluentSolution.valuesOf(). */
export type VariableSelection =
    | NumericVar
    | BoolVar
    | readonly VariableSelection[]
    | { readonly [key: string]: VariableSelection };

/** Maps fluent variable handles to their primitive solution value types. */
export type ValuesOf<T> =
    T extends BoolVar ? boolean
    : T extends NumericVar ? number
    : T extends readonly unknown[] ? { -readonly [K in keyof T]: ValuesOf<T[K]> }
    : T extends Record<string, unknown> ? { -readonly [K in keyof T]: ValuesOf<T[K]> }
    : never;

function isTaggedValue(value: unknown): value is RawTaggedValue {
    if (typeof value !== "object" || value === null) return false;
    const candidate = value as { type?: unknown; value?: unknown };
    if (candidate.type === "Bool") return typeof candidate.value === "boolean";
    return (
        (candidate.type === "Int" || candidate.type === "Real") &&
        typeof candidate.value === "number"
    );
}

function parseRawSolution(value: unknown): RawSolution {
    if (typeof value !== "object" || value === null) {
        throw new ModelBuilderError(
            "The solver returned an invalid solution object",
            "solving",
        );
    }
    const candidate = value as Partial<RawSolution>;
    if (!Array.isArray(candidate.assignment) || typeof candidate.value !== "number") {
        throw new ModelBuilderError(
            "The solver returned an invalid assignment or objective value",
            "solving",
        );
    }
    if (
        !(candidate.constraints instanceof Map) &&
        (typeof candidate.constraints !== "object" || candidate.constraints === null)
    ) {
        throw new ModelBuilderError(
            "The solver returned invalid constraint activity",
            "solving",
        );
    }
    return candidate as RawSolution;
}

/** A solved fluent model with primitive and recursively typed value readback. */
export class FluentSolution {
    readonly #owner: symbol;
    readonly #objectiveValue: number;
    readonly #assignments = new Map<string, number | RawTaggedValue>();
    readonly #constraints = new Map<string, number>();

    /** @internal */
    constructor(owner: symbol, rawValue: unknown) {
        const raw = parseRawSolution(rawValue);
        this.#owner = owner;
        this.#objectiveValue = raw.value;

        for (const assignment of raw.assignment) {
            if (
                typeof assignment !== "object" ||
                assignment === null ||
                typeof assignment.name !== "string" ||
                (typeof assignment.value !== "number" && !isTaggedValue(assignment.value))
            ) {
                throw new ModelBuilderError(
                    "The solver returned an invalid variable assignment",
                    "solving",
                );
            }
            this.#assignments.set(assignment.name, assignment.value);
        }

        const entries = raw.constraints instanceof Map
            ? raw.constraints.entries()
            : Object.entries(raw.constraints);
        for (const [name, activity] of entries) {
            if (!name.startsWith(INTERNAL_PREFIX) && typeof activity === "number") {
                this.#constraints.set(name, activity);
            }
        }
    }

    /** Returns the solved objective value. */
    value(): number {
        return this.#objectiveValue;
    }

    /** Reads a handle as a primitive Boolean or number. */
    valueOf<V extends BoolVar | NumericVar>(
        variable: V,
    ): V extends BoolVar ? boolean : number {
        if (!(variable instanceof BoolVar) && !(variable instanceof NumericVar)) {
            throw new ModelBuilderError(
                "FluentSolution.valueOf() expects a variable handle",
                "construction",
            );
        }
        this.#assertOwner(expressionOwner(variable));
        return this.#resolve({
            id: variable.id,
            name: variable.name,
            sourceName: variable.name,
            kind: variable instanceof BoolVar ? "boolean" : "number",
            owner: this.#owner,
        }) as V extends BoolVar ? boolean : number;
    }

    /** Recursively maps handles in an array or object to primitive values. */
    valuesOf<T extends VariableSelection>(selection: T): ValuesOf<T> {
        return this.#mapSelection(selection) as ValuesOf<T>;
    }

    /** Evaluates a fluent expression using this solution's assignments. */
    eval<E extends NumericExpression | BooleanExpression>(
        expression: E,
    ): ExpressionValue<E> {
        if (!(expression instanceof NumericExpression)) {
            throw new ModelBuilderError(
                "FluentSolution.eval() expects a fluent expression",
                "construction",
            );
        }
        this.#assertOwner(expressionOwner(expression));
        return evaluateExpression(expression, (variable) => this.#resolve(variable));
    }

    /** Returns the activity for a user-named constraint, if present. */
    constraintValue(name: string): number | undefined {
        if (name.startsWith(INTERNAL_PREFIX)) return undefined;
        return this.#constraints.get(name);
    }

    #mapSelection(selection: VariableSelection): unknown {
        if (selection instanceof BoolVar || selection instanceof NumericVar) {
            return this.valueOf(selection);
        }
        if (Array.isArray(selection)) {
            return selection.map((value) => this.#mapSelection(value));
        }
        if (typeof selection === "object" && selection !== null) {
            const result: Record<string, unknown> = {};
            for (const [key, value] of Object.entries(selection)) {
                result[key] = this.#mapSelection(value);
            }
            return result;
        }
        throw new ModelBuilderError(
            "FluentSolution.valuesOf() received an unsupported value",
            "construction",
        );
    }

    #resolve(variable: VariableReference): number | boolean {
        this.#assertOwner(variable.owner);
        if (!this.#assignments.has(variable.name)) {
            throw new ModelBuilderError(
                `The solver did not return an assignment for variable '${variable.name}'`,
                "construction",
            );
        }
        const value = this.#assignments.get(variable.name)!;
        if (variable.kind === "boolean") {
            if (isTaggedValue(value) && value.type === "Bool") return value.value;
            throw new ModelBuilderError(
                `Variable '${variable.name}' expected a Boolean solver value`,
                "construction",
            );
        }
        if (typeof value === "number") return value;
        if (value.type === "Int" || value.type === "Real") return value.value;
        throw new ModelBuilderError(
            `Variable '${variable.name}' expected a numeric solver value`,
            "construction",
        );
    }

    #assertOwner(owner?: symbol): void {
        if (owner !== undefined && owner !== this.#owner) {
            throw new ModelBuilderError(
                "Cannot read an expression or variable from a different model",
                "construction",
            );
        }
    }
}
