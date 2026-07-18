export { ModelBuilderError } from "./error.js";
export type {
    ModelBuilderErrorDetails,
    ModelBuilderStage,
} from "./error.js";

export {
    BoolVar,
    BooleanExpression,
    Constraint,
    NumericExpression,
    NumericVar,
    abs,
    all,
    any,
    max,
    min,
    sum,
} from "./expressions.js";
export type {
    BooleanInput,
    ConstraintRelation,
    ExpressionValue,
    NumericInput,
} from "./expressions.js";

export { ModelBuilder, VariableDefinition } from "./model.js";
export type {
    AnyVariableDefinition,
    SolveOptions,
    SolverName,
    VariableFromDefinition,
    VariablesFromDefinitions,
} from "./model.js";

export { FluentSolution } from "./solution.js";
export type {
    ValuesOf,
    VariableSelection,
} from "./solution.js";
