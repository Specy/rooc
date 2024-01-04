import {
    RoocParser as _RoocParser,
    Problem as _Problem,
    CompilationError as _CompilationError,
} from './pkg'
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
    getSpan() {
        return this.instance.get_span_wasm();
    }
    getErrorKind() {
        return this.instance.get_kind_wasm();
    }
    serialize() {
        return this.instance.serialize_wasm();
    }
    message() {
        if (this.source) {
            this.instance.to_string_from_source_wasm(this.source);
        } else {
            this.instance.to_error_string_wasm();
        }
    }
}

export class Problem {
    instance: _Problem;
    constructor(instance: _Problem) {
        this.instance = instance;
    }
    
}