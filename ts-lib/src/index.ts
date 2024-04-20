import {
    CompilationError as _CompilationError,
    InputSpan,
    LinearModel,
    Model as _Model,
    OptimalTableau,
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
    StandardLinearModel,
    Tableau,
    TransformErrorWrapper as _TransformErrorWrapper,
    WasmPipableData,
    WasmPipeError,
    WasmPipeRunner
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
type RoocData =
    RoocType<PipeDataType.String, string> |
    RoocType<PipeDataType.Parser, RoocParser> |
    RoocType<PipeDataType.PreModel, PreModel> |
    RoocType<PipeDataType.Model, Model> |
    RoocType<PipeDataType.LinearModel, LinearModel> |
    RoocType<PipeDataType.StandardLinearModel, StandardLinearModel> |
    RoocType<PipeDataType.Tableau, Tableau> |
    RoocType<PipeDataType.OptimalTableau, OptimalTableau>

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
            return {type: PipeDataType.LinearModel, data: data.to_linear_model()}
        case PipeDataType.StandardLinearModel:
            return {type: PipeDataType.StandardLinearModel, data: data.to_standard_linear_model()}
        case PipeDataType.Tableau:
            return {type: PipeDataType.Tableau, data: data.to_tableau()}
        case PipeDataType.OptimalTableau:
            return {type: PipeDataType.OptimalTableau, data: data.to_optimal_tableau()}

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
    VariableType,
    SerializedVariableKind,
    SerializedTransformError,
    SerializedTokenType,
    SerializedTypedToken,
    SerializedPrimitiveKind,
    BlockFunctionKind,
    BlockScopedFunctionKind,
    UnOp,
    BinOp,
    Comparison,
    OptimizationType,
    ParseError,
    SerializedVariableToAssert,
    SerializedVariablesDomainDeclaration,
} from './pkg/rooc'