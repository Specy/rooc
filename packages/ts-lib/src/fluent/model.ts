import { ModelBuilderError } from "./error.js";
import { Pipes, WasmPipeError, WasmPipeRunner } from "../pkg/rooc.js";
import {
    BoolVar,
    BooleanExpression,
    Constraint,
    NumericExpression,
    NumericVar,
    asNumericExpression,
    constraintOwner,
    constraintState,
    constraintVariableIds,
    expressionOwner,
    expressionVariableIds,
    formatRoocNumber,
    serializeConstraint,
    serializeExpression,
    type NumericInput,
    type VariableReference,
} from "./expressions.js";
import { FluentSolution } from "./solution.js";

type VariableKind = "boolean" | "number";

type BooleanDomain = { readonly type: "boolean" };
type IntegerDomain = {
    readonly type: "integer";
    readonly min: number;
    readonly max: number;
};
type RealDomain =
    | { readonly type: "real" }
    | { readonly type: "real"; readonly min: number; readonly max: number };
type NonNegativeDomain =
    | { readonly type: "nonNegative" }
    | {
        readonly type: "nonNegative";
        readonly min: number;
        readonly max: number;
    };

type VariableDomain =
    | BooleanDomain
    | IntegerDomain
    | RealDomain
    | NonNegativeDomain;

type VariableDefinitionState = {
    readonly kind: VariableKind;
    readonly domain: VariableDomain;
    readonly count?: number;
};

const variableDefinitionStates = new WeakMap<object, VariableDefinitionState>();

function variableDefinitionState(
    definition: VariableDefinition<VariableKind, boolean>,
): VariableDefinitionState {
    const state = variableDefinitionStates.get(definition);
    if (!state) {
        throw new ModelBuilderError(
            "Invalid fluent variable definition",
            "declaration",
        );
    }
    return state;
}

/** An immutable scalar or array declaration used by ModelBuilder.var() or vars(). */
export class VariableDefinition<
    K extends "boolean" | "number",
    IsArray extends boolean = false,
> {
    private declare readonly definitionKind: K;
    private declare readonly arrayDefinition: IsArray;

    /** @internal */
    constructor(
        kind: K,
        domain: unknown,
        count?: number,
    ) {
        variableDefinitionStates.set(this, {
            kind,
            domain: domain as VariableDomain,
            count,
        });
    }

    /** Creates an array declaration whose members use `<name>_<index>`. */
    array(
        this: VariableDefinition<K, false>,
        count: number,
    ): VariableDefinition<K, true> {
        const state = variableDefinitionState(this);
        if (state.count !== undefined) {
            throw new ModelBuilderError(
                "Variable array definitions cannot be nested",
                "declaration",
            );
        }
        if (!Number.isSafeInteger(count) || count < 0) {
            throw new ModelBuilderError(
                `Variable array count must be a non-negative safe integer, got ${String(count)}`,
                "declaration",
            );
        }
        return new VariableDefinition(state.kind as K, state.domain, count);
    }
}

export type AnyVariableDefinition =
    | VariableDefinition<"boolean", boolean>
    | VariableDefinition<"number", boolean>;

export type VariableFromDefinition<D extends AnyVariableDefinition> =
    D extends VariableDefinition<"boolean", true> ? BoolVar[]
    : D extends VariableDefinition<"boolean", false> ? BoolVar
    : D extends VariableDefinition<"number", true> ? NumericVar[]
    : D extends VariableDefinition<"number", false> ? NumericVar
    : never;

export type VariablesFromDefinitions<
    D extends Record<string, AnyVariableDefinition>,
> = {
    [K in keyof D]: VariableFromDefinition<D[K]>;
};

type Declaration = {
    readonly id: number;
    readonly name: string;
    readonly sourceName: string;
    readonly kind: VariableKind;
    readonly domain: VariableDomain;
    readonly handle: BoolVar | NumericVar;
};

type ModelConstraint = {
    readonly value: Constraint | BooleanExpression;
    readonly name?: string;
};

type Objective = {
    readonly type: "max" | "min";
    readonly expression: NumericExpression;
};

export type SolverName = "auto" | "microlp" | "clarabel";

export type SolveOptions = {
    readonly solver?: SolverName;
};

const I32_MIN = -2_147_483_648;
const I32_MAX = 2_147_483_647;
const INTERNAL_PREFIX = "$__rooc_ts_";
const INTERNAL_KEEP_NAME = `${INTERNAL_PREFIX}keep`;

// This mirrors parser keywords and built-in names that the transformer rejects.
const RESERVED_NAMES = new Set([
    "abs",
    "all",
    "and",
    "any",
    "as",
    "avg",
    "conjunction",
    "define",
    "difference",
    "disjunction",
    "E",
    "edges",
    "else",
    "enum",
    "enumerate",
    "exclusive_disjunction",
    "false",
    "for",
    "Graph",
    "if",
    "iff",
    "implies",
    "in",
    "intersection",
    "Infinity",
    "len",
    "let",
    "max",
    "min",
    "MinusInfinity",
    "N",
    "N_of",
    "neigh_edges",
    "neigh_edges_of",
    "nodes",
    "not",
    "or",
    "PI",
    "prod",
    "range",
    "s.t.",
    "solve",
    "sum",
    "true",
    "union",
    "V",
    "where",
    "xor",
    "zip",
]);

function finiteBound(value: number, label: string): number {
    if (!Number.isFinite(value)) {
        throw new ModelBuilderError(
            `${label} must be finite, got ${String(value)}`,
            "declaration",
        );
    }
    return Object.is(value, -0) ? 0 : value;
}

function orderedBounds(min: number, max: number): [number, number] {
    const finiteMin = finiteBound(min, "Minimum bound");
    const finiteMax = finiteBound(max, "Maximum bound");
    if (finiteMin > finiteMax) {
        throw new ModelBuilderError(
            `Variable bounds are invalid: minimum ${finiteMin} is greater than maximum ${finiteMax}`,
            "declaration",
        );
    }
    return [finiteMin, finiteMax];
}

function validateName(name: string, kind: "variable" | "constraint"): void {
    if (name.startsWith(INTERNAL_PREFIX)) {
        throw new ModelBuilderError(
            `${kind} name '${name}' uses the reserved ${INTERNAL_PREFIX} prefix`,
            "declaration",
        );
    }
    if (!/^\$?_*[A-Za-z][A-Za-z0-9_]*$/.test(name)) {
        throw new ModelBuilderError(
            `Invalid ${kind} name '${name}'; expected a ROOC identifier`,
            "declaration",
        );
    }
    if (RESERVED_NAMES.has(name)) {
        throw new ModelBuilderError(
            `${kind} name '${name}' is reserved by ROOC`,
            "declaration",
        );
    }
}

function sourceIdentifier(name: string): string {
    return /^\$?_*[A-Za-z][A-Za-z0-9]*$/.test(name) ? name : `\\${name}`;
}

function domainSource(domain: VariableDomain): string {
    switch (domain.type) {
        case "boolean":
            return "Boolean";
        case "integer":
            return `IntegerRange(${formatRoocNumber(domain.min)}, ${formatRoocNumber(domain.max)})`;
        case "real":
            return "min" in domain
                ? `Real(${formatRoocNumber(domain.min)}, ${formatRoocNumber(domain.max)})`
                : "Real";
        case "nonNegative":
            return "min" in domain
                ? `NonNegativeReal(${formatRoocNumber(domain.min)}, ${formatRoocNumber(domain.max)})`
                : "NonNegativeReal";
    }
}

/**
 * Builds a typed optimization model and serializes it through ROOC's existing
 * parser, linearizer, and solvers.
 */
export class ModelBuilder {
    readonly #owner = Symbol("ROOC ModelBuilder owner");
    readonly #declarations: Declaration[] = [];
    readonly #variableNames = new Set<string>();
    readonly #constraints: ModelConstraint[] = [];
    readonly #constraintNames = new Set<string>();
    #objective?: Objective;

    /** Declares a Boolean 0/1 variable domain. */
    bool(): VariableDefinition<"boolean", false> {
        return new VariableDefinition("boolean", { type: "boolean" });
    }

    /** Declares an integer variable domain with inclusive i32 bounds. */
    int(min: number, max: number): VariableDefinition<"number", false> {
        if (!Number.isSafeInteger(min) || !Number.isSafeInteger(max)) {
            throw new ModelBuilderError(
                "Integer bounds must be safe integers",
                "declaration",
            );
        }
        if (min < I32_MIN || max > I32_MAX) {
            throw new ModelBuilderError(
                `Integer bounds must fit the supported i32 range ${I32_MIN}..${I32_MAX}`,
                "declaration",
            );
        }
        const [orderedMin, orderedMax] = orderedBounds(min, max);
        return new VariableDefinition("number", {
            type: "integer",
            min: orderedMin,
            max: orderedMax,
        });
    }

    /** Declares an unbounded or explicitly bounded real variable domain. */
    real(): VariableDefinition<"number", false>;
    real(min: number, max: number): VariableDefinition<"number", false>;
    real(min?: number, max?: number): VariableDefinition<"number", false> {
        if (min === undefined && max === undefined) {
            return new VariableDefinition("number", { type: "real" });
        }
        if (min === undefined || max === undefined) {
            throw new ModelBuilderError(
                "Bounded real variables require both minimum and maximum bounds",
                "declaration",
            );
        }
        const [orderedMin, orderedMax] = orderedBounds(min, max);
        return new VariableDefinition("number", {
            type: "real",
            min: orderedMin,
            max: orderedMax,
        });
    }

    /** Declares an unbounded-above or explicitly bounded non-negative real. */
    nonNegative(): VariableDefinition<"number", false>;
    nonNegative(min: number, max: number): VariableDefinition<"number", false>;
    nonNegative(min?: number, max?: number): VariableDefinition<"number", false> {
        if (min === undefined && max === undefined) {
            return new VariableDefinition("number", { type: "nonNegative" });
        }
        if (min === undefined || max === undefined) {
            throw new ModelBuilderError(
                "Bounded non-negative variables require both minimum and maximum bounds",
                "declaration",
            );
        }
        const [orderedMin, orderedMax] = orderedBounds(min, max);
        if (orderedMin < 0) {
            throw new ModelBuilderError(
                "A non-negative variable cannot have a negative minimum bound",
                "declaration",
            );
        }
        return new VariableDefinition("number", {
            type: "nonNegative",
            min: orderedMin,
            max: orderedMax,
        });
    }

    /** Declares one variable or a generated array of variables. */
    var<const D extends AnyVariableDefinition>(
        name: string,
        definition: D,
    ): VariableFromDefinition<D> {
        validateName(name, "variable");
        if (!(definition instanceof VariableDefinition)) {
            throw new ModelBuilderError(
                "ModelBuilder.var() expects a fluent variable definition",
                "declaration",
            );
        }
        const definitionState = variableDefinitionState(definition);
        const count = definitionState.count;
        if (
            count !== undefined &&
            (!Number.isSafeInteger(count) || count < 0)
        ) {
            throw new ModelBuilderError(
                `Variable array count must be a non-negative safe integer, got ${String(count)}`,
                "declaration",
            );
        }
        const names =
            count === undefined
                ? [name]
                : Array.from({ length: count }, (_, index) => `${name}_${index}`);

        for (const generatedName of names) {
            validateName(generatedName, "variable");
            if (this.#variableNames.has(generatedName)) {
                throw new ModelBuilderError(
                    `Variable '${generatedName}' already exists in this model`,
                    "declaration",
                );
            }
        }

        const handles = names.map((generatedName) =>
            this.#declare(
                generatedName,
                definitionState.kind,
                definitionState.domain,
            ),
        );
        return (count === undefined ? handles[0] : handles) as VariableFromDefinition<D>;
    }

    /** Declares a record of variables while preserving every input key and type. */
    vars<const D extends Record<string, AnyVariableDefinition>>(
        definitions: D,
    ): VariablesFromDefinitions<D> {
        const result: Partial<Record<keyof D, BoolVar | NumericVar | (BoolVar | NumericVar)[]>> = {};
        for (const key of Object.keys(definitions) as (keyof D & string)[]) {
            Object.defineProperty(result, key, {
                configurable: true,
                enumerable: true,
                value: this.var(key, definitions[key]),
                writable: true,
            });
        }
        return result as VariablesFromDefinitions<D>;
    }

    /** Adds a numeric comparison or a Boolean expression that must be true. */
    with(
        value: Constraint | BooleanExpression,
        name?: string,
    ): this {
        if (!(value instanceof Constraint) && !(value instanceof BooleanExpression)) {
            throw new ModelBuilderError(
                "ModelBuilder.with() expects a constraint or Boolean expression",
                "construction",
            );
        }
        this.#assertOwner(
            value instanceof Constraint ? constraintOwner(value) : expressionOwner(value),
        );

        const embeddedName = value instanceof Constraint
            ? constraintState(value).name
            : undefined;
        if (name !== undefined && embeddedName !== undefined && name !== embeddedName) {
            throw new ModelBuilderError(
                `Constraint is already named '${embeddedName}' and cannot also be named '${name}'`,
                "declaration",
            );
        }
        const resolvedName = name ?? embeddedName;
        if (resolvedName !== undefined) {
            validateName(resolvedName, "constraint");
            if (this.#constraintNames.has(resolvedName)) {
                throw new ModelBuilderError(
                    `Constraint '${resolvedName}' already exists in this model`,
                    "declaration",
                );
            }
            this.#constraintNames.add(resolvedName);
        }
        this.#constraints.push({ value, name: resolvedName });
        return this;
    }

    /** Adds every constraint or Boolean assertion from an iterable. */
    withAll(values: Iterable<Constraint | BooleanExpression>): this {
        for (const value of values) this.with(value);
        return this;
    }

    /** Sets or replaces the maximization objective. */
    maximize(expression: NumericInput): this {
        return this.#setObjective("max", expression);
    }

    /** Sets or replaces the minimization objective. */
    minimize(expression: NumericInput): this {
        return this.#setObjective("min", expression);
    }

    /** Clears the objective and makes this a feasibility model. */
    satisfy(): this {
        this.#objective = undefined;
        return this;
    }

    /** Serializes the complete typed model to deterministic ROOC source. */
    toRooc(): string {
        const lines: string[] = [];
        const usedVariableIds = new Set<number>();

        if (this.#objective) {
            lines.push(
                `${this.#objective.type} ${serializeExpression(this.#objective.expression)}`,
            );
            for (const id of expressionVariableIds(this.#objective.expression)) {
                usedVariableIds.add(id);
            }
        } else {
            lines.push("solve");
        }
        lines.push("s.t.");

        for (const constraint of this.#constraints) {
            const body = constraint.value instanceof Constraint
                ? serializeConstraint(constraint.value)
                : serializeExpression(constraint.value);
            const prefix = constraint.name
                ? `${sourceIdentifier(constraint.name)}: `
                : "";
            lines.push(`    ${prefix}${body}`);

            const ids = constraint.value instanceof Constraint
                ? constraintVariableIds(constraint.value)
                : expressionVariableIds(constraint.value);
            for (const id of ids) usedVariableIds.add(id);
        }

        const unused = this.#declarations.filter(
            (declaration) => !usedVariableIds.has(declaration.id),
        );
        const keepDeclarations = unused.length > 0
            ? unused
            : this.#constraints.length === 0 && this.#declarations.length > 0
              ? [this.#declarations[0]]
              : [];
        if (keepDeclarations.length > 0) {
            const keepBody = keepDeclarations
                .map((declaration) => `0 * ${declaration.sourceName}`)
                .join(" + ");
            lines.push(
                `    ${sourceIdentifier(INTERNAL_KEEP_NAME)}: ${keepBody} = 0`,
            );
        }

        if (this.#declarations.length > 0) {
            lines.push("define");
            for (const declaration of this.#declarations) {
                lines.push(
                    `    ${declaration.sourceName} as ${domainSource(declaration.domain)}`,
                );
            }
        }

        // The parser requires a newline after an otherwise-empty s.t. section.
        return lines.length === 2 ? `${lines.join("\n")}\n` : lines.join("\n");
    }

    /** Solves the current model synchronously through an existing WASM solver. */
    solve(options: SolveOptions = {}): FluentSolution {
        const solver = options.solver ?? "auto";
        const solverPipe = (() => {
            switch (solver) {
                case "auto": return Pipes.AutoSolverPipe;
                case "microlp": return Pipes.MILPSolverPipe;
                case "clarabel": return Pipes.RealPipe;
                default:
                    throw new ModelBuilderError(
                        `Unknown fluent solver '${String(solver)}'`,
                        "construction",
                    );
            }
        })();
        const source = this.toRooc();
        const runner = WasmPipeRunner.new_wasm();
        runner.add_step_by_name(Pipes.CompilerPipe);
        runner.add_step_by_name(Pipes.PreModelPipe);
        runner.add_step_by_name(Pipes.ModelPipe);
        runner.add_step_by_name(Pipes.LinearModelPipe);
        runner.add_step_by_name(solverPipe);

        try {
            const results = runner.wasm_run_from_string(source, [], []);
            const solved = results[results.length - 1];
            if (!solved) {
                throw new ModelBuilderError(
                    "The fluent solver pipeline returned no solution",
                    "solving",
                    { source, context: results },
                );
            }
            const raw = solver === "clarabel"
                ? solved.to_real_solution()
                : solved.to_milp_solution();
            return new FluentSolution(this.#owner, raw);
        } catch (error) {
            if (error instanceof ModelBuilderError) throw error;
            if (error instanceof WasmPipeError) {
                const message = error.wasm_get_error();
                const context = error.wasm_to_context();
                const stage = context.length >= 5
                    ? "solving"
                    : context.length >= 4
                      ? "linearization"
                      : "compilation";
                throw new ModelBuilderError(message, stage, {
                    source,
                    cause: error,
                    context,
                });
            }
            throw new ModelBuilderError(
                error instanceof Error ? error.message : String(error),
                "solving",
                { source, cause: error },
            );
        }
    }

    #declare(
        name: string,
        kind: VariableKind,
        domain: VariableDomain,
    ): BoolVar | NumericVar {
        const id = this.#declarations.length;
        const reference: VariableReference = {
            id,
            name,
            sourceName: sourceIdentifier(name),
            kind,
            owner: this.#owner,
        };
        const handle = kind === "boolean"
            ? new BoolVar(reference)
            : new NumericVar(reference);
        this.#variableNames.add(name);
        this.#declarations.push({ ...reference, domain, handle });
        return handle;
    }

    #setObjective(type: "max" | "min", input: NumericInput): this {
        const expression = asNumericExpression(input);
        this.#assertOwner(expressionOwner(expression));
        this.#objective = { type, expression };
        return this;
    }

    #assertOwner(owner?: symbol): void {
        if (owner !== undefined && owner !== this.#owner) {
            throw new ModelBuilderError(
                "Cannot use an expression from a different model",
                "construction",
            );
        }
    }
}
