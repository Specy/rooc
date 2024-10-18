import {
    CompilationError as _CompilationError,
    EqualityConstraint as _EqualityConstraint,
    InputSpan,
    LinearConstraint as _LinearConstraint,
    LinearModel as _LinearModel,
    Model as _Model,
    OptimalTableau as _OptimalTableau,
    OptimalTableauWithSteps as _OptimalTableauWithSteps,
    ParseError,
    PipeDataType,
    Pipes,
    PreModel as _PreModel,
    RoocParser as _RoocParser,
    SerializedCompilationError,
    SerializedModel,
    SerializedPreModel,
    SerializedTransformError,
    SerializedTypedToken,
    SimplexStep as _SimplexStep,
    StandardLinearModel as _StandardLinearModel,
    Tableau as _Tableau,
    TransformErrorWrapper as _TransformErrorWrapper,
    WasmPipableData,
    WasmPipeError,
    WasmPipeRunner,
} from './pkg/rooc.js'
import {Err, Ok, Result} from 'ts-results'

export class RoocParser {
    instance: _RoocParser;
    source: string;

    constructor(source: string) {
        this.instance = _RoocParser.new_wasm(source);
        this.source = source;
    }

    static fromParser(parser: _RoocParser) {
        return new RoocParser(parser.wasm_get_source())
    }

    format(): Result<string, CompilationError> {
        try {
            return Ok(this.instance.format_wasm())
        } catch (e) {
            return Err(new CompilationError(e, this.source))
        }
    }

    compile(): Result<PreModel, CompilationError> {
        try {
            return Ok(new PreModel(this.instance.parse_wasm(), this.source))
        } catch (e) {
            return Err(new CompilationError(e, this.source))
        }
    }

    compileAndTransform(): Result<Model, string> {
        try {
            return Ok(new Model(this.instance.parse_and_transform_wasm()))
        } catch (e) {
            return Err(e)
        }

    }
}

export class CompilationError {
    instance: _CompilationError;
    source?: string;

    constructor(instance: _CompilationError, source?: string) {
        this.instance = instance;
        this.source = source;
    }

    getSpan(): InputSpan {
        return this.instance.get_span_wasm();
    }

    getErrorKind(): ParseError {
        return this.instance.get_kind_wasm();
    }

    serialize(): SerializedCompilationError {
        return this.instance.serialize_wasm();
    }

    message() {
        if (this.source) {
            return this.instance.to_string_from_source_wasm(this.source);
        } else {
            return this.instance.to_error_string_wasm();
        }
    }
}

export class PreModel {
    instance: _PreModel
    source: string

    constructor(instance: _PreModel, source: string) {
        this.instance = instance;
        this.source = source;
    }

    static fromPreModel(preModel: _PreModel) {
        return new PreModel(preModel, preModel.wasm_get_source())
    }

    serialize(): SerializedPreModel {
        return this.instance.serialize_wasm()
    }

    transform(): Result<Model, TransformError> {
        try {
            return Ok(new Model(this.instance.transform_wasm()))
        } catch (e) {
            return Err(new TransformError(e, this.source))
        }
    }

    typeCheck(): Result<null, TransformError> {
        try {
            this.instance.type_check_wasm()
            return Ok(null)
        } catch (e) {
            return Err(new TransformError(e, this.source))
        }
    }

    createTypeMap(): Map<number, SerializedTypedToken> {
        return this.instance.create_token_type_map_wasm()
    }

    toLatex(): string {
        return this.instance.to_latex_wasm()
    }

    format(): string {
        return this.instance.format_wasm()
    }
}

type RoocType<T, D> = {
    type: T
    data: D
}
export type RoocData =
    RoocType<PipeDataType.String, string> |
    RoocType<PipeDataType.Parser, RoocParser> |
    RoocType<PipeDataType.PreModel, PreModel> |
    RoocType<PipeDataType.Model, Model> |
    RoocType<PipeDataType.LinearModel, LinearModel> |
    RoocType<PipeDataType.StandardLinearModel, StandardLinearModel> |
    RoocType<PipeDataType.Tableau, SimplexTableau> |
    RoocType<PipeDataType.OptimalTableau, OptimalTableau> |
    RoocType<PipeDataType.OptimalTableauWithSteps, OptimalTableauWithSteps> |
    RoocType<PipeDataType.BinarySolution, BinaryIntegerSolution<boolean>> | 
    RoocType<PipeDataType.BinarySolution, BinaryIntegerSolution<VarValue>>

function toRoocData(data: WasmPipableData): RoocData {
    switch (data.wasm_get_type()) {
        case PipeDataType.String:
            return {type: PipeDataType.String, data: data.to_string_data()}
        case PipeDataType.Parser:
            return {type: PipeDataType.Parser, data: RoocParser.fromParser(data.to_parser())}
        case PipeDataType.PreModel:
            return {type: PipeDataType.PreModel, data: PreModel.fromPreModel(data.to_pre_model())}
        case PipeDataType.Model:
            return {type: PipeDataType.Model, data: new Model(data.to_model())}
        case PipeDataType.LinearModel:
            return {type: PipeDataType.LinearModel, data: new LinearModel(data.to_linear_model())}
        case PipeDataType.StandardLinearModel:
            return {
                type: PipeDataType.StandardLinearModel,
                data: new StandardLinearModel(data.to_standard_linear_model())
            }
        case PipeDataType.Tableau:
            return {type: PipeDataType.Tableau, data: new SimplexTableau(data.to_tableau())}
        case PipeDataType.OptimalTableau:
            return {type: PipeDataType.OptimalTableau, data: new OptimalTableau(data.to_optimal_tableau())}
        case PipeDataType.OptimalTableauWithSteps:
            return {
                type: PipeDataType.OptimalTableauWithSteps,
                data: new OptimalTableauWithSteps(data.to_optimal_tableau_with_steps())
            }
        case PipeDataType.BinarySolution:
            return {
                type: PipeDataType.BinarySolution,
                data: data.to_binary_solution()
            }
    }
}


export class LinearModel {
    instance: _LinearModel
    private cache: {
        variables?: string[]
        objectiveCoefficients?: number[]
        objectiveOffset?: number
        constraints?: LinearConstraint[]
    } = {}

    constructor(instance: _LinearModel) {
        this.instance = instance;
    }

    stringify() {
        return this.instance.wasm_to_string()
    }

    getVariables() {
        return this.cache.variables ??= this.instance.wasm_get_variables()
    }

    getObjectiveCoefficients() {
        return this.cache.objectiveCoefficients ??= Array.from(this.instance.wasm_get_objective())
    }

    getObjectiveOffset() {
        return this.instance.wasm_get_objective_offset()
    }

    getOptimizationType() {
        return this.instance.wasm_get_optimization_type()
    }

    getConstraints() {

        return this.cache.constraints ??= Array.from(this.instance.wasm_get_constraints()).map(c => new LinearConstraint(c))
    }
}

export class LinearConstraint {
    instance: _LinearConstraint
    private cache: {
        coefficients?: number[]
    } = {}

    constructor(instance: _LinearConstraint) {
        this.instance = instance;
    }

    getCoefficients() {
        return this.cache.coefficients ??= Array.from(this.instance.wasm_get_coefficients())
    }

    getRhs() {
        return this.instance.wasm_get_rhs()
    }

    getConstraintType() {
        return this.instance.wasm_get_constraint_type()
    }

}

export class SimplexTableau {
    instance: _Tableau
    private cache: {
        aMatrix?: number[][]
        bVector?: number[]
        cVector?: number[]
        variableNames?: string[]
        indexesOfVarsInBasis?: number[]
    } = {}

    constructor(instance: _Tableau) {
        this.instance = instance;
    }

    getOffsetValue() {
        return this.instance.wasm_get_value_offset()
    }

    getCurrentValue() {
        return this.instance.wasm_get_current_value()
    }

    getAMatrix(): number[][] {
        return this.cache.aMatrix ??= this.instance.wasm_get_a()
    }


    getBVector() {
        return this.cache.bVector ??= Array.from(this.instance.wasm_get_b())
    }

    getCVector() {
        return this.cache.cVector ??= Array.from(this.instance.wasm_get_c())
    }

    getVariableNames() {
        return this.cache.variableNames ??= this.instance.wasm_get_variables()
    }

    getIndexesOfVarsInBasis() {
        return this.cache.indexesOfVarsInBasis ??= Array.from(this.instance.wasm_get_in_basis())
    }

    step(variableIndexesToAvoid?: number[]) {
        const vars = new Uint32Array(variableIndexesToAvoid ?? [])
        this.instance.wasm_step(vars)
        this.cache = {}
    }

    stringify() {
        return this.instance.wasm_to_string()
    }

}

export class OptimalTableau {
    instance: _OptimalTableau
    private cache: {
        tableau?: SimplexTableau
        variablesValues?: number[]
    } = {}

    constructor(instance: _OptimalTableau) {
        this.instance = instance;
    }

    getTableau() {
        return this.cache.tableau ??= new SimplexTableau(this.instance.wasm_get_tableau())
    }

    getVariablesValues() {

        return this.cache.variablesValues ??= Array.from(this.instance.wasm_get_variables_values())
    }

    getOptimalValue() {
        return this.instance.wasm_get_optimal_value()
    }
}

export class SimplexStep {
    instance: _SimplexStep
    private cache: {
        tableau?: SimplexTableau
    } = {}

    constructor(instance: _SimplexStep) {
        this.instance = instance;
    }

    getPivot() {
        return {
            entering: this.instance.wasm_get_leaving(),
            leaving: this.instance.wasm_get_entering()
        }
    }

    getTableau() {
        return this.cache.tableau ??= new SimplexTableau(this.instance.wasm_get_tableau())
    }

}

export class OptimalTableauWithSteps {
    instance: _OptimalTableauWithSteps
    private cache: {
        result?: OptimalTableau
        steps?: SimplexStep[]
    } = {}

    constructor(instance: _OptimalTableauWithSteps) {
        this.instance = instance
    }

    getResult() {
        return this.cache.result ??= new OptimalTableau(this.instance.wasm_get_result())
    }

    getSteps() {
        return this.cache.steps ??= Array.from(this.instance.wasm_get_steps()).map(s => new SimplexStep(s))
    }
}


export class StandardLinearModel {
    instance: _StandardLinearModel
    private cache: {
        objective?: number[]
        aMatrix?: number[][]
        constraints?: EqualityConstraint[]
        variables?: string[]
        cVector?: number[]
        bVector?: number[]
    } = {}

    constructor(instance: _StandardLinearModel) {
        this.instance = instance;
    }

    getObjective() {
        return this.cache.objective ??= Array.from(this.instance.wasm_get_objective())

    }

    getConstraints() {
        return this.cache.constraints ??= Array.from(this.instance.wasm_get_constraints()).map(c => new EqualityConstraint(c))
    }

    getVariables() {
        return this.cache.variables ??= this.instance.wasm_get_variables()
    }

    getAMatrix(): number[][] {
        return this.cache.aMatrix ??= this.instance.wasm_get_a()
    }

    getCVector() {
        return this.cache.cVector ??= Array.from(this.instance.wasm_get_c())
    }

    getBVector() {
        return this.cache.bVector ??= Array.from(this.instance.wasm_get_b())
    }

    isObjectiveFlipped() {
        return this.instance.wasm_get_flip_objective()
    }

    stringify() {
        return this.instance.wasm_to_string()
    }
}

export class EqualityConstraint {
    instance: _EqualityConstraint
    private cache: {
        coefficients?: number[]
    } = {}

    constructor(instance: _EqualityConstraint) {

        this.instance = instance;
    }

    getCoefficients() {
        return this.cache.coefficients ??= Array.from(this.instance.wasm_get_coefficients())
    }

    getRhs() {
        return this.instance.wasm_get_rhs()
    }

}

export class RoocRunnablePipe {
    instance: WasmPipeRunner

    constructor(steps: Pipes[]) {
        this.instance = WasmPipeRunner.new_wasm(steps)
    }

    run(source: string): Result<RoocData[], { error: String, context: RoocData[] }> {
        try {
            const data = this.instance.wasm_run_from_string(source)
            return Ok(data.map(toRoocData))
        } catch (e) {
            if (e instanceof WasmPipeError) {
                return Err({error: e.wasm_get_error(), context: e.wasm_to_context().map(toRoocData)})
            }
            throw e
        }

    }
}


export class TransformError {
    instance: _TransformErrorWrapper
    source?: string

    constructor(instance: _TransformErrorWrapper, source?: string) {
        this.instance = instance;
        this.source = source;
    }

    serialize(): SerializedTransformError {
        return this.instance.serialize_wasm()
    }

    getMessageFromSource(source: string): string {
        return this.instance.get_error_from_source(source)
    }

    getTracedError(): string {
        return this.instance.get_traced_error()
    }

    getOriginSpan(): InputSpan | undefined {
        return this.instance.get_origin_span()
    }

    getBaseError(): SerializedTransformError {
        return this.instance.get_base_error()
    }

    stringifyBaseError(): string {
        return this.instance.stringify_base_error()
    }

    message() {
        try {
            if (this.source) {
                try {
                    return this.instance.get_error_from_source(this.source);
                } catch (e) {
                    console.error(`Error while getting error from source`, e, this.source, this.getOriginSpan())
                    return this.instance.get_traced_error()
                }
            } else {
                return this.instance.get_traced_error();
            }
        } catch (e) {
            console.error(e)
        }
        try {
            const span = this.getOriginSpan()
            if (span) {
                return `Error at line ${span.start_line}:${span.start_column}`
            }
        } catch (e) {
            console.error(e)
        }
        return `Unknown error`
    }

    getTrace(): InputSpan[] {
        return this.instance.get_trace();
    }
}

export class Model {
    instance: _Model;

    constructor(instance: _Model) {
        this.instance = instance;
    }

    serialize(): SerializedModel {
        return this.instance.serialize_wasm()
    }

    stringify(): string {
        return this.instance.to_string_wasm()
    }
}

export type VarValue = {
    type: 'integer'
    value: number
} | {
    type: 'binary'
    value: boolean
}

export type BinaryAssignment<T> = {
    name: string
    value: T
}

export type BinaryIntegerSolution<T> = {
    assignment: BinaryAssignment<T>[]
    value: number
}


export * from './runtime'


export type {
    SerializedAddressableAccess,
    SerializedBlockFunction,
    SerializedBlockScopedFunction,
    SerializedCompilationError,
    SerializedCompoundVariable,
    SerializedFunctionCall,
    SerializedCondition,
    SerializedConstant,
    SerializedExp,
    SerializedGraph,
    SerializedGraphEdge,
    SerializedGraphNode,
    SerializedIterable,
    SerializedIterableSet,
    SerializedObjective,
    SerializedPreExp,
    SerializedPreConstraint,
    SerializedPreObjective,
    SerializedPreModel,
    SerializedModel,
    SerializedPrimitive,
    SerializedSpanned,
    SerializedTuple,
    SerializedVariableKind,
    SerializedTransformError,
    SerializedTokenType,
    SerializedTypedToken,
    SerializedPrimitiveKind,
    ParseError,
    SerializedVariableToAssert,
    SerializedVariablesDomainDeclaration,
} from './pkg/rooc'

export {
    VariableType,
    BlockFunctionKind,
    BlockScopedFunctionKind,
    UnOp,
    BinOp,
    Comparison,
    OptimizationType,

} from './pkg/rooc'