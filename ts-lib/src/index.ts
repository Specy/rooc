import {
    RoocParser as _RoocParser,
    Problem as _Problem,
    CompilationError as _CompilationError,
    SerializedProblem,
    SerializedCompilationError,
    ParseError,
    InputSpan,
} from './pkg/rooc'
import { Ok, Err, Result } from 'ts-results'
export class RoocParser {
    instance: _RoocParser;
    source: string;
    constructor(source: string) {
        this.instance = _RoocParser.new_wasm(source);
        this.source = source;
    }
    verify(): Result<null, string> {
        try {
            this.instance.verify_wasm();
            Ok(null)
        } catch (e) {
            return Err(e)
        }
    }
    format(): Result<string, CompilationError> {
        try {
            return Ok(this.instance.format_wasm())
        } catch (e) {
            return Err(new CompilationError(e, this.source))
        }
    }
    compile(): Result<Problem, CompilationError> {
        try {
            return Ok(new Problem(this.instance.parse_and_transform_wasm()))
        } catch (e) {
            return Err(new CompilationError(e, this.source))
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
    getErrorKind(): ParseError{
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
} from './pkg/rooc'