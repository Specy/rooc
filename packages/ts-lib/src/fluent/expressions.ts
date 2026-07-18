import { ModelBuilderError } from "./error.js";

export type NumericInput = number | NumericExpression;
export type BooleanInput = boolean | BooleanExpression;
export type ExpressionValue<E> = E extends BooleanExpression ? boolean : number;

export type VariableValueKind = "boolean" | "number";
export type VariableReference = {
    readonly id: number;
    readonly name: string;
    readonly sourceName: string;
    readonly kind: VariableValueKind;
    readonly owner: symbol;
};

type ArithmeticOperator = "add" | "sub" | "mul" | "div";
type LogicOperator = "and" | "or" | "xor" | "implies" | "iff";
type ExtremeOperator = "min" | "max";
export type ConstraintRelation = "=" | "<=" | "<" | ">=" | ">";

type ExpressionNode =
    | { readonly type: "number"; readonly value: number }
    | { readonly type: "boolean"; readonly value: boolean }
    | { readonly type: "variable"; readonly variable: VariableReference }
    | {
        readonly type: "arithmetic";
        readonly operator: ArithmeticOperator;
        readonly lhs: ExpressionNode;
        readonly rhs: ExpressionNode;
    }
    | {
        readonly type: "logic";
        readonly operator: LogicOperator;
        readonly lhs: ExpressionNode;
        readonly rhs: ExpressionNode;
    }
    | {
        readonly type: "neg" | "not" | "abs";
        readonly operand: ExpressionNode;
    }
    | {
        readonly type: "extreme";
        readonly operator: ExtremeOperator;
        readonly operands: readonly ExpressionNode[];
    }
    | { readonly type: "sum"; readonly operands: readonly ExpressionNode[] }
    | {
        readonly type: "all" | "any";
        readonly operands: readonly ExpressionNode[];
    };

type ExpressionState = {
    readonly node: ExpressionNode;
    readonly owner?: symbol;
};

type ConstraintState = {
    readonly lhs: NumericExpression;
    readonly relation: ConstraintRelation;
    readonly rhs: NumericExpression;
    readonly owner?: symbol;
    readonly name?: string;
};

const expressionStates = new WeakMap<NumericExpression, ExpressionState>();
const constraintStates = new WeakMap<Constraint, ConstraintState>();

function expressionState(expression: NumericExpression): ExpressionState {
    const state = expressionStates.get(expression);
    if (!state) {
        throw new ModelBuilderError(
            "Invalid fluent expression instance",
            "construction",
        );
    }
    return state;
}

export function constraintState(constraint: Constraint): ConstraintState {
    const state = constraintStates.get(constraint);
    if (!state) {
        throw new ModelBuilderError(
            "Invalid fluent constraint instance",
            "construction",
        );
    }
    return state;
}

function combineOwners(lhs?: symbol, rhs?: symbol): symbol | undefined {
    if (lhs !== undefined && rhs !== undefined && lhs !== rhs) {
        throw new ModelBuilderError(
            "Cannot combine expressions from a different model",
            "construction",
        );
    }
    return lhs ?? rhs;
}

function finiteNumber(value: number): number {
    if (!Number.isFinite(value)) {
        throw new ModelBuilderError(
            `Expected a finite number, got ${String(value)}`,
            "construction",
        );
    }
    return Object.is(value, -0) ? 0 : value;
}

function numericInput(input: NumericInput): NumericExpression {
    if (typeof input === "number") {
        return makeNumeric({ type: "number", value: finiteNumber(input) });
    }
    if (!(input instanceof NumericExpression)) {
        throw new ModelBuilderError(
            "Expected a number or numeric expression",
            "construction",
        );
    }
    return input;
}

function booleanInput(input: BooleanInput): BooleanExpression {
    if (typeof input === "boolean") {
        return makeBoolean({ type: "boolean", value: input });
    }
    if (!(input instanceof BooleanExpression)) {
        throw new ModelBuilderError(
            "Expected a Boolean expression",
            "construction",
        );
    }
    return input;
}

function makeNumeric(node: ExpressionNode, owner?: symbol): NumericExpression {
    return new NumericValueExpression(node, owner);
}

function makeBoolean(node: ExpressionNode, owner?: symbol): BooleanExpression {
    return new BooleanValueExpression(node, owner);
}

/** A typed arithmetic expression in a fluent optimization model. */
export class NumericExpression {
    /** @internal */
    protected constructor(node: unknown, owner?: symbol) {
        expressionStates.set(this, { node: node as ExpressionNode, owner });
    }

    add(rhs: NumericInput): NumericExpression {
        return this.arithmetic("add", rhs);
    }

    sub(rhs: NumericInput): NumericExpression {
        return this.arithmetic("sub", rhs);
    }

    mul(factor: number): NumericExpression {
        return this.arithmetic("mul", finiteNumber(factor));
    }

    div(divisor: number): NumericExpression {
        const value = finiteNumber(divisor);
        if (value === 0) {
            throw new ModelBuilderError(
                "Cannot divide a fluent expression by zero",
                "construction",
            );
        }
        return this.arithmetic("div", value);
    }

    neg(): NumericExpression {
        const state = expressionState(this);
        return makeNumeric({ type: "neg", operand: state.node }, state.owner);
    }

    eq(rhs: NumericInput): Constraint {
        return this.compare("=", rhs);
    }

    le(rhs: NumericInput): Constraint {
        return this.compare("<=", rhs);
    }

    lt(rhs: NumericInput): Constraint {
        return this.compare("<", rhs);
    }

    ge(rhs: NumericInput): Constraint {
        return this.compare(">=", rhs);
    }

    gt(rhs: NumericInput): Constraint {
        return this.compare(">", rhs);
    }

    private arithmetic(operator: ArithmeticOperator, rhs: NumericInput): NumericExpression {
        const lhsState = expressionState(this);
        const right = numericInput(rhs);
        const rhsState = expressionState(right);
        return makeNumeric(
            {
                type: "arithmetic",
                operator,
                lhs: lhsState.node,
                rhs: rhsState.node,
            },
            combineOwners(lhsState.owner, rhsState.owner),
        );
    }

    private compare(relation: ConstraintRelation, rhs: NumericInput): Constraint {
        return new Constraint(this, relation, numericInput(rhs));
    }
}

/** A typed Boolean expression that can also be used arithmetically as 0/1. */
export class BooleanExpression extends NumericExpression {
    /** @internal */
    protected constructor(node: unknown, owner?: symbol) {
        super(node, owner);
    }

    and(rhs: BooleanInput): BooleanExpression {
        return this.logic("and", rhs);
    }

    or(rhs: BooleanInput): BooleanExpression {
        return this.logic("or", rhs);
    }

    xor(rhs: BooleanInput): BooleanExpression {
        return this.logic("xor", rhs);
    }

    implies(rhs: BooleanInput): BooleanExpression {
        return this.logic("implies", rhs);
    }

    iff(rhs: BooleanInput): BooleanExpression {
        return this.logic("iff", rhs);
    }

    not(): BooleanExpression {
        const state = expressionState(this);
        return makeBoolean({ type: "not", operand: state.node }, state.owner);
    }

    private logic(operator: LogicOperator, rhs: BooleanInput): BooleanExpression {
        const lhsState = expressionState(this);
        const right = booleanInput(rhs);
        const rhsState = expressionState(right);
        return makeBoolean(
            {
                type: "logic",
                operator,
                lhs: lhsState.node,
                rhs: rhsState.node,
            },
            combineOwners(lhsState.owner, rhsState.owner),
        );
    }
}

class NumericValueExpression extends NumericExpression {
    constructor(node: ExpressionNode, owner?: symbol) {
        super(node, owner);
    }
}

class BooleanValueExpression extends BooleanExpression {
    constructor(node: ExpressionNode, owner?: symbol) {
        super(node, owner);
    }
}

/** A numeric variable handle declared by a ModelBuilder. */
export class NumericVar extends NumericExpression {
    readonly id: number;
    readonly name: string;

    /** @internal */
    constructor(value: unknown) {
        const reference = value as VariableReference;
        super({ type: "variable", variable: reference }, reference.owner);
        this.id = reference.id;
        this.name = reference.name;
    }
}

/** A Boolean variable handle declared by a ModelBuilder. */
export class BoolVar extends BooleanExpression {
    readonly id: number;
    readonly name: string;

    /** @internal */
    constructor(value: unknown) {
        const reference = value as VariableReference;
        super({ type: "variable", variable: reference }, reference.owner);
        this.id = reference.id;
        this.name = reference.name;
    }
}

/** A numeric comparison that can be added to a ModelBuilder. */
export class Constraint {
    /** @internal */
    constructor(
        lhs: NumericExpression,
        relation: ConstraintRelation,
        rhs: NumericExpression,
        name?: string,
    ) {
        const lhsState = expressionState(lhs);
        const rhsState = expressionState(rhs);
        constraintStates.set(this, {
            lhs,
            relation,
            rhs,
            owner: combineOwners(lhsState.owner, rhsState.owner),
            name,
        });
    }

    named(name: string): Constraint {
        const state = constraintState(this);
        return new Constraint(state.lhs, state.relation, state.rhs, name);
    }
}

export function abs(expr: NumericInput): NumericExpression {
    const expression = numericInput(expr);
    const state = expressionState(expression);
    return makeNumeric({ type: "abs", operand: state.node }, state.owner);
}

export function min(exprs: Iterable<NumericInput>): NumericExpression {
    return extreme("min", exprs);
}

export function max(exprs: Iterable<NumericInput>): NumericExpression {
    return extreme("max", exprs);
}

function extreme(
    operator: ExtremeOperator,
    exprs: Iterable<NumericInput>,
): NumericExpression {
    const expressions = Array.from(exprs, numericInput);
    if (expressions.length === 0) {
        throw new ModelBuilderError(
            `${operator} requires at least one expression; the input was empty`,
            "construction",
        );
    }
    let owner: symbol | undefined;
    const operands = expressions.map((expression) => {
        const state = expressionState(expression);
        owner = combineOwners(owner, state.owner);
        return state.node;
    });
    return makeNumeric({ type: "extreme", operator, operands }, owner);
}

export function sum(exprs: Iterable<NumericInput>): NumericExpression {
    const expressions = Array.from(exprs, numericInput);
    let owner: symbol | undefined;
    const operands = expressions.map((expression) => {
        const state = expressionState(expression);
        owner = combineOwners(owner, state.owner);
        return state.node;
    });
    return makeNumeric({ type: "sum", operands }, owner);
}

export function all(exprs: Iterable<BooleanInput>): BooleanExpression {
    return booleanAggregation("all", exprs);
}

export function any(exprs: Iterable<BooleanInput>): BooleanExpression {
    return booleanAggregation("any", exprs);
}

function booleanAggregation(
    type: "all" | "any",
    exprs: Iterable<BooleanInput>,
): BooleanExpression {
    const expressions = Array.from(exprs, booleanInput);
    if (expressions.length === 0) {
        return makeBoolean({ type: "boolean", value: type === "all" });
    }
    let owner: symbol | undefined;
    const operands = expressions.map((expression) => {
        const state = expressionState(expression);
        owner = combineOwners(owner, state.owner);
        return state.node;
    });
    return makeBoolean({ type, operands }, owner);
}

/** @internal */
export function expressionOwner(expression: NumericExpression): symbol | undefined {
    return expressionState(expression).owner;
}

/** @internal */
export function constraintOwner(constraint: Constraint): symbol | undefined {
    return constraintState(constraint).owner;
}

/** @internal */
export function asNumericExpression(input: NumericInput): NumericExpression {
    return numericInput(input);
}

/** @internal */
export function formatRoocNumber(value: number): string {
    const normalized = finiteNumber(value);
    const source = String(normalized);
    if (!/[eE]/.test(source)) return source;

    const [coefficient, exponentText] = source.toLowerCase().split("e");
    const exponent = Number(exponentText);
    const negative = coefficient.startsWith("-");
    const unsigned = negative ? coefficient.slice(1) : coefficient;
    const [integerPart, fractionalPart = ""] = unsigned.split(".");
    const digits = integerPart + fractionalPart;
    const decimalIndex = integerPart.length + exponent;
    let expanded: string;
    if (decimalIndex <= 0) {
        expanded = `0.${"0".repeat(-decimalIndex)}${digits}`;
    } else if (decimalIndex >= digits.length) {
        expanded = `${digits}${"0".repeat(decimalIndex - digits.length)}`;
    } else {
        expanded = `${digits.slice(0, decimalIndex)}.${digits.slice(decimalIndex)}`;
    }
    return negative ? `-${expanded}` : expanded;
}

function serializeNode(node: ExpressionNode): string {
    switch (node.type) {
        case "number":
            return formatRoocNumber(node.value);
        case "boolean":
            return node.value ? "true" : "false";
        case "variable":
            return node.variable.sourceName;
        case "arithmetic": {
            const operators: Record<ArithmeticOperator, string> = {
                add: "+",
                sub: "-",
                mul: "*",
                div: "/",
            };
            return `(${serializeNode(node.lhs)} ${operators[node.operator]} ${serializeNode(node.rhs)})`;
        }
        case "logic":
            return `(${serializeNode(node.lhs)} ${node.operator} ${serializeNode(node.rhs)})`;
        case "neg":
            return `(-${serializeNode(node.operand)})`;
        case "not":
            return `(not ${serializeNode(node.operand)})`;
        case "abs":
            return `abs { ${serializeNode(node.operand)} }`;
        case "extreme":
            return `${node.operator} { ${node.operands.map(serializeNode).join(", ")} }`;
        case "sum": {
            if (node.operands.length === 0) return "0";
            return node.operands
                .map(serializeNode)
                .reduce((lhs, rhs) => `(${lhs} + ${rhs})`);
        }
        case "all":
        case "any":
            return `${node.type} { ${node.operands.map(serializeNode).join(", ")} }`;
    }
}

/** @internal */
export function serializeExpression(expression: NumericExpression): string {
    return serializeNode(expressionState(expression).node);
}

/** @internal */
export function serializeConstraint(constraint: Constraint): string {
    const state = constraintState(constraint);
    return `${serializeExpression(state.lhs)} ${state.relation} ${serializeExpression(state.rhs)}`;
}

function collectVariableIds(node: ExpressionNode, ids: Set<number>): void {
    switch (node.type) {
        case "variable":
            ids.add(node.variable.id);
            return;
        case "arithmetic":
        case "logic":
            collectVariableIds(node.lhs, ids);
            collectVariableIds(node.rhs, ids);
            return;
        case "neg":
        case "not":
        case "abs":
            collectVariableIds(node.operand, ids);
            return;
        case "extreme":
        case "sum":
        case "all":
        case "any":
            for (const operand of node.operands) collectVariableIds(operand, ids);
            return;
        case "number":
        case "boolean":
            return;
    }
}

/** @internal */
export function expressionVariableIds(expression: NumericExpression): ReadonlySet<number> {
    const ids = new Set<number>();
    collectVariableIds(expressionState(expression).node, ids);
    return ids;
}

/** @internal */
export function constraintVariableIds(constraint: Constraint): ReadonlySet<number> {
    const state = constraintState(constraint);
    const ids = new Set<number>(expressionVariableIds(state.lhs));
    for (const id of expressionVariableIds(state.rhs)) ids.add(id);
    return ids;
}

function numericValue(value: number | boolean): number {
    return typeof value === "boolean" ? Number(value) : value;
}

function booleanValue(value: number | boolean): boolean {
    return typeof value === "boolean" ? value : value !== 0;
}

function evaluateNode(
    node: ExpressionNode,
    resolve: (variable: VariableReference) => number | boolean,
): number | boolean {
    switch (node.type) {
        case "number":
        case "boolean":
            return node.value;
        case "variable":
            return resolve(node.variable);
        case "arithmetic": {
            const lhs = numericValue(evaluateNode(node.lhs, resolve));
            const rhs = numericValue(evaluateNode(node.rhs, resolve));
            switch (node.operator) {
                case "add": return lhs + rhs;
                case "sub": return lhs - rhs;
                case "mul": return lhs * rhs;
                case "div": return lhs / rhs;
            }
        }
        case "logic": {
            const lhs = booleanValue(evaluateNode(node.lhs, resolve));
            const rhs = booleanValue(evaluateNode(node.rhs, resolve));
            switch (node.operator) {
                case "and": return lhs && rhs;
                case "or": return lhs || rhs;
                case "xor": return lhs !== rhs;
                case "implies": return !lhs || rhs;
                case "iff": return lhs === rhs;
            }
        }
        case "neg":
            return -numericValue(evaluateNode(node.operand, resolve));
        case "not":
            return !booleanValue(evaluateNode(node.operand, resolve));
        case "abs":
            return Math.abs(numericValue(evaluateNode(node.operand, resolve)));
        case "extreme": {
            const values = node.operands.map((operand) =>
                numericValue(evaluateNode(operand, resolve))
            );
            return node.operator === "min" ? Math.min(...values) : Math.max(...values);
        }
        case "sum":
            return node.operands.reduce(
                (total, operand) => total + numericValue(evaluateNode(operand, resolve)),
                0,
            );
        case "all":
            return node.operands.every((operand) =>
                booleanValue(evaluateNode(operand, resolve))
            );
        case "any":
            return node.operands.some((operand) =>
                booleanValue(evaluateNode(operand, resolve))
            );
    }
}

/** @internal */
export function evaluateExpression<
    E extends NumericExpression | BooleanExpression,
>(
    expression: E,
    resolve: (variable: VariableReference) => number | boolean,
): ExpressionValue<E> {
    return evaluateNode(expressionState(expression).node, resolve) as ExpressionValue<E>;
}
