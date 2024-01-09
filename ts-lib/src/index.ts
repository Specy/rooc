import {
    RoocParser as _RoocParser,
    Problem as _Problem,
    CompilationError as _CompilationError,
    SerializedProblem,
    SerializedCompilationError,
    ParseError,
    InputSpan,
    PreProblem as _PreProblem,
    SerializedPreProblem,
    TransformErrorWrapper as _TransformErrorWrapper,
    SerializedTransformError,
    SerializedTypedToken,
} from './pkg/rooc.js'
import { Ok, Err, Result } from 'ts-results'
export class RoocParser {
    instance: _RoocParser;
    source: string;
    constructor(source: string) {
        this.instance = _RoocParser.new_wasm(source);
        this.source = source;
    }
    format(): Result<string, CompilationError> {
        try {
            return Ok(this.instance.format_wasm())
        } catch (e) {
            return Err(new CompilationError(e, this.source))
        }
    }
    compile(): Result<PreProblem, CompilationError> {
        try {
            return Ok(new PreProblem(this.instance.parse_wasm(), this.source))
        } catch (e) {
            return Err(new CompilationError(e, this.source))
        }
    }
    compileAndTransform(): Result<Problem, string> {
        try {
            return Ok(new Problem(this.instance.parse_and_transform_wasm()))
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
export class PreProblem {
    instance: _PreProblem
    source: string
    constructor(instance: _PreProblem, source: string) {
        this.instance = instance;
        this.source = source;
    }
    serialize(): SerializedPreProblem {
        return this.instance.serialize_wasm()
    }
    transform(): Result<Problem, TransformError> {
        try {
            return Ok(new Problem(this.instance.transform_wasm()))
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
    format(): string {
        return this.instance.format_wasm()
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

export class Problem {
    instance: _Problem;
    constructor(instance: _Problem) {
        this.instance = instance;
    }
    serialize(): SerializedProblem {
        return this.instance.serialize_wasm()
    }
    stringify(): string {
        return this.instance.to_string_wasm()
    }
}


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
    SerializedPreCondition,
    SerializedPreObjective,
    SerializedPreProblem,
    SerializedProblem,
    SerializedPrimitive,
    SerializedSpanned,
    SerializedTuple,
    SerializedVariableType,
    SerializedTransformError
} from './pkg/rooc'