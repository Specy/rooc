import {
    CompilationError as _CompilationError,
    EqualityConstraint as _EqualityConstraint,
    InputSpan,
    JsFunction,
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
    SerializedPrimitive,
    SerializedPrimitiveKind,
    SerializedTransformError,
    SerializedTypedToken,
    SimplexStep as _SimplexStep,
    StandardLinearModel as _StandardLinearModel,
    Tableau as _Tableau,
    TransformErrorWrapper as _TransformErrorWrapper,
    WasmPipableData,
    WasmPipeError,
    WasmPipeRunner
} from './pkg/rooc.js'
import {Err, Ok, Result} from 'ts-results-es'
import {ExtractArgTypes, ExtractReturnArgs, PrimitiveKind} from "./runtime";


export type ReturnCallback<T extends [string, SerializedPrimitiveKind][]> = ((args: ExtractReturnArgs<T>, staticArgs: ExtractArgTypes<T>) => SerializedPrimitiveKind)

type MakeRoocFunction<T extends [string, SerializedPrimitiveKind][]> = {
    name: string,
    parameters: T,
    returns: SerializedPrimitiveKind | ReturnCallback<NoInfer<T>>
    call: (...args: ExtractArgTypes<NoInfer<T>>) => SerializedPrimitive,
    type_checker?: (...args: SerializedPrimitiveKind[]) => null | string
    description?: string
}


/**
 * Create a RoocFunction, this function can be provided to the RoocParser to be used in the transformation process
 * @param name the name of the function
 * @param parameters the type of parameters of the rooc function
 * @param returns the type of the return value of the rooc function, or a function that will be called to determine the return type
 * @param type_checker a function that will be called to check the types of parameters, this disabled the default type checking and this will be used instead
 * @param call the function that will be called when the rooc function is called
 * @param description a description of the function
 */
export function makeRoocFunction<const T extends [string, SerializedPrimitiveKind][]>({
                                                                                          name,
                                                                                          parameters,
                                                                                          returns,
                                                                                          type_checker,
                                                                                          call,
                                                                                          description
                                                                                      }: MakeRoocFunction<T>) {
    return new RoocFunction<T>(
        JsFunction.new(
            (...args) => {
                try {
                    // @ts-ignore
                    return call(...args)
                } catch (e) {
                    throw String(e)
                }
            },
            name,
            parameters,
            typeof returns === 'function' ? (...args) => {
                try {
                    // @ts-ignore
                    return returns(...args)
                } catch (e) {
                    throw String(e)
                }
            } : returns,
            type_checker
        ),
        name,
        parameters,
        returns,
        description
    )
}




/**
 * Create a RoocFunction, this function can be provided to the RoocParser to be used in the transformation process
 * @param name the name of the function
 * @param parameters the type of parameters of the rooc function
 * @param returns the type of the return value of the rooc function
 * @param call the function that will be called when the rooc function is called
 * @param description a description of the function
 */
export class RoocFunction<T extends [string, SerializedPrimitiveKind][] = [string, SerializedPrimitiveKind][]> {
    instance: JsFunction
    name: string
    description?: string
    parameters: T
    returns: SerializedPrimitiveKind | ReturnCallback<NoInfer<T>>

    constructor(
        instance: JsFunction,
        name: string,
        parameters: T,
        returns: SerializedPrimitiveKind | ReturnCallback<NoInfer<T>>,
        description?: string
    ) {
        this.instance = instance
        this.name = name
        this.parameters = parameters
        this.returns = returns
        this.description = description
    }
}


/**
 * The RoocParser is the main entry point to the Rooc library, it allows to parse, transform and compile rooc code
 */
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

    /**
     * Formats the source code
     */
    format(): Result<string, CompilationError> {
        try {
            return Ok(this.instance.format_wasm())
        } catch (e) {
            return Err(new CompilationError(e, this.source))
        }
    }

    /**
     * Compiles the source code into a PreModel, this is the first step in the compilation process.
     * It only parses the source code and does not transform it.
     */
    compile(): Result<PreModel, CompilationError> {
        try {
            return Ok(new PreModel(this.instance.parse_wasm(), this.source))
        } catch (e) {
            return Err(new CompilationError(e, this.source))
        }
    }

    /**
     * Compiles the source code into a Model
     * @param fns additional functions that can be used in the transformation process
     */
    compileAndTransform(fns: RoocFunction[] = []): Result<Model, string> {
        try {
            return Ok(new Model(this.instance.parse_and_transform_wasm(cloneJsFunction(fns))))
        } catch (e) {
            return Err(e)
        }

    }
}

function cloneJsFunction(fns: RoocFunction[]) {
    return fns.map(f => f.instance.clone_wasm())
}

export class CompilationError {
    instance: _CompilationError;
    source?: string;

    constructor(instance: _CompilationError, source?: string) {
        this.instance = instance;
        this.source = source;
    }

    /**
     * Get the span of the error, this can be used to highlight the error in the source code
     */
    getSpan(): InputSpan {
        return this.instance.get_span_wasm();
    }

    /**
     * Get the kind of the error that occurred
     */
    getErrorKind(): ParseError {
        return this.instance.get_kind_wasm();
    }

    /**
     * Serialize the error as a json object
     */
    serialize(): SerializedCompilationError {
        return this.instance.serialize_wasm();
    }

    /**
     * Format the error message including a stack trace
     */
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

    /**
     * Serialize the PreModel as a json object
     */
    serialize(): SerializedPreModel {
        return this.instance.serialize_wasm()
    }

    /**
     * Transforms the PreModel into a Model
     * @param fns additional functions that can be used in the transformation process
     */
    transform(fns: RoocFunction[] = []): Result<Model, TransformError> {
        try {
            return Ok(new Model(this.instance.transform_wasm(cloneJsFunction(fns))))
        } catch (e) {
            return Err(new TransformError(e, this.source))
        }
    }

    /**
     * Executes type checking on the PreModel, reporting any type errors that are found
     * @param fns additional functions that can be used in the type checking process
     */
    typeCheck(fns: RoocFunction[] = []): Result<null, TransformError> {
        try {
            this.instance.type_check_wasm(cloneJsFunction(fns))
            return Ok(null)
        } catch (e) {
            return Err(new TransformError(e, this.source))
        }
    }

    /**
     * Create a type map for the source code provided, the key is the offset of the token in the source code
     * and the type is the type of the token
     * @param fns
     */
    createTypeMap(fns: RoocFunction[] = []): Map<number, SerializedTypedToken> {
        return this.instance.create_token_type_map_wasm(cloneJsFunction(fns))
    }

    /**
     * Converts the PreModel into a latex string
     */
    toLatex(): string {
        return this.instance.to_latex_wasm()
    }

    /**
     * Formats the source code
     */
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
    RoocType<PipeDataType.IntegerBinarySolution, BinaryIntegerSolution<VarValue>>

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
        case PipeDataType.IntegerBinarySolution:
            return {
                type: PipeDataType.IntegerBinarySolution,
                data: data.to_integer_binary_solution()
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

    /**
     * Stringifies back the linear model into a string
     */
    stringify() {
        return this.instance.wasm_to_string()
    }

    /**
     * Get all the variable names of the linear model
     */
    getVariables() {
        return this.cache.variables ??= this.instance.wasm_get_variables()
    }

    /**
     * Get the coefficients of the objective function
     */
    getObjectiveCoefficients() {
        return this.cache.objectiveCoefficients ??= Array.from(this.instance.wasm_get_objective())
    }

    /**
     * Get the offset of the objective function, this is the constant term of the objective function
     */
    getObjectiveOffset() {
        return this.instance.wasm_get_objective_offset()
    }

    /**
     * Get which kind of optimization is being done
     */
    getOptimizationType() {
        return this.instance.wasm_get_optimization_type()
    }

    /**
     * Get all the constraints of the linear model
     */
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

    /**
     * Get the coefficients of the constraint
     */
    getCoefficients() {
        return this.cache.coefficients ??= Array.from(this.instance.wasm_get_coefficients())
    }

    /**
     * Get the right hand side of the constraint, this is the constant term of the constraint
     */
    getRhs() {
        return this.instance.wasm_get_rhs()
    }

    /**
     * Get the type of the constraint
     */
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

    /**
     * Get the offset value of the tableau
     */
    getOffsetValue() {
        return this.instance.wasm_get_value_offset()
    }

    /**
     * Get the current value of the tableau
     */
    getCurrentValue() {
        return this.instance.wasm_get_current_value()
    }

    /**
     * Get the A matrix of the tableau, this is the matrix of the coefficients of the constraints
     */
    getAMatrix(): number[][] {
        return this.cache.aMatrix ??= this.instance.wasm_get_a()
    }

    /**
     * Get the B vector of the tableau, this is the right hand side of the constraints
     */
    getBVector() {
        return this.cache.bVector ??= Array.from(this.instance.wasm_get_b())
    }

    /**
     * Get the C vector of the tableau, this is the coefficients of the objective function
     */
    getCVector() {
        return this.cache.cVector ??= Array.from(this.instance.wasm_get_c())
    }

    /**
     * Get the variable names of the tableau
     */
    getVariableNames() {
        return this.cache.variableNames ??= this.instance.wasm_get_variables()
    }

    /**
     * Get the indexes of the variables in the basis, the index is relative to the variable names
     */
    getIndexesOfVarsInBasis() {
        return this.cache.indexesOfVarsInBasis ??= Array.from(this.instance.wasm_get_in_basis())
    }

    /**
     * Step the tableau to the next iteration
     * @param variableIndexesToAvoid the indexes of the variables to avoid stepping onto
     */
    step(variableIndexesToAvoid?: number[]) {
        const vars = new Uint32Array(variableIndexesToAvoid ?? [])
        this.instance.wasm_step(vars)
        this.cache = {}
    }

    /**
     * Convert the tableau back to a string
     */
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

    /**
     * Get the tableau of the optimal solution
     */
    getTableau() {
        return this.cache.tableau ??= new SimplexTableau(this.instance.wasm_get_tableau())
    }

    /**
     * Get the values of the variables of the optimal solution
     */
    getVariablesValues() {

        return this.cache.variablesValues ??= Array.from(this.instance.wasm_get_variables_values())
    }

    /**
     * Get the optimal value of the solution
     */
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

    /**
     * Get the pivot decision of the step
     */
    getPivot() {
        return {
            entering: this.instance.wasm_get_leaving(),
            leaving: this.instance.wasm_get_entering()
        }
    }

    /**
     * Get the tableau of the step
     */
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

    /**
     * Get the optimal tableu
     */
    getResult() {
        return this.cache.result ??= new OptimalTableau(this.instance.wasm_get_result())
    }

    /**
     * Get the steps of the simplex algorithm
     */
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

    /**
     * Get the objective function of the standard linear model, this is the coefficients of the objective function
     */
    getObjective() {
        return this.cache.objective ??= Array.from(this.instance.wasm_get_objective())

    }

    /**
     * Get the equality constraints of the standard linear model
     */
    getConstraints() {
        return this.cache.constraints ??= Array.from(this.instance.wasm_get_constraints()).map(c => new EqualityConstraint(c))
    }

    /**
     * Get the list of variables of the standard linear model
     */
    getVariables() {
        return this.cache.variables ??= this.instance.wasm_get_variables()
    }

    /**
     * Get the A matrix of the standard linear model, this is the matrix of the coefficients of the constraints
     */
    getAMatrix(): number[][] {
        return this.cache.aMatrix ??= this.instance.wasm_get_a()
    }

    /**
     * Get the C vector of the standard linear model, this is the coefficients of the objective function
     */
    getCVector() {
        return this.cache.cVector ??= Array.from(this.instance.wasm_get_c())
    }

    /**
     * Get the B vector of the standard linear model, this is the right hand side of the constraints
     */
    getBVector() {
        return this.cache.bVector ??= Array.from(this.instance.wasm_get_b())
    }

    /**
     * Get if the objective function has been flipped from the conversion process
     */
    isObjectiveFlipped() {
        return this.instance.wasm_get_flip_objective()
    }

    /**
     * Convert the standard linear model back to a string
     */
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

    /**
     * Get the coefficients of the constraint
     */
    getCoefficients() {
        return this.cache.coefficients ??= Array.from(this.instance.wasm_get_coefficients())
    }

    /**
     * Get the right hand side of the constraint, this is the constant term of the constraint
     */
    getRhs() {
        return this.instance.wasm_get_rhs()
    }

}

/**
 * The RoocRunnablePipe allows to run a series of pipes on a source code, each pipe transforms the data from one type to another
 * The output of the previous pipe is the input of the next pipe
 */
export class RoocRunnablePipe {
    instance: WasmPipeRunner

    constructor(steps: Pipes[]) {
        this.instance = WasmPipeRunner.new_wasm(steps)
    }

    /**
     * Run the pipe on a source code
     * @param source the source code to run the pipe on
     * @param fns additional functions that can be used in the transformation process
     * @returns each step of the pipe returns some data, if there was an error, it's previous context is returned too
     */
    run(source: string, fns: RoocFunction[] = []): Result<RoocData[], { error: String, context: RoocData[] }> {
        try {
            const data = this.instance.wasm_run_from_string(source, cloneJsFunction(fns))
            return Ok(data.map(toRoocData))
        } catch (e) {
            if (e instanceof WasmPipeError) {
                return Err({error: e.wasm_get_error(), context: e.wasm_to_context().map(toRoocData)})
            }
            throw e
        }

    }
}

/**
 *  The TransformError is an error that occurs during the transformation process of a PreModel into a Model
 */
export class TransformError {
    instance: _TransformErrorWrapper
    source?: string

    constructor(instance: _TransformErrorWrapper, source?: string) {
        this.instance = instance;
        this.source = source;
    }

    /**
     * Serialize the error as a json object
     */
    serialize(): SerializedTransformError {
        return this.instance.serialize_wasm()
    }

    /**
     * Get the error message from the source code
     * @param source
     */
    getMessageFromSource(source: string): string {
        return this.instance.get_error_from_source(source)
    }

    /**
     * Converts the error to a string, including the stack trace
     */
    getTracedError(): string {
        return this.instance.get_traced_error()
    }

    /**
     * Get the span of the original error, if available
     */
    getOriginSpan(): InputSpan | undefined {
        return this.instance.get_origin_span()
    }

    /**
     * Get the base error of the transform error
     */
    getBaseError(): SerializedTransformError {
        return this.instance.get_base_error()
    }

    /**
     * Stringify the base error, this is the error without the stack trace
     */
    stringifyBaseError(): string {
        return this.instance.stringify_base_error()
    }

    /**
     * Get the error message
     */
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

    /**
     * Serialize the model as a json object
     */
    serialize(): SerializedModel {
        return this.instance.serialize_wasm()
    }

    /**
     * Stringify the model back to a string
     */
    stringify(): string {
        return this.instance.to_string_wasm()
    }
}

export type VarValue = {
    type: 'Int'
    value: number
} | {
    type: 'Int'
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