use core::fmt;
use std::{fmt::Debug, ops::Deref, ops::DerefMut};

#[allow(unused_imports)]
use crate::prelude::*;
use pest::{Span, iterators::Pair};
use serde::{Deserialize, Serialize};

use crate::parser::pre_model::Rule;

/// Represents a span of text in the input source, tracking location information.
///
/// This struct stores the starting line, column, absolute position and length of a span of text,
/// along with a flag indicating if the span has been modified.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct InputSpan {
    //as u32 as realistically we won't have more than 4 billion characters in a file
    pub start_line: u32,
    pub start_column: u32,
    pub start: u32,
    pub len: u32,
    pub tempered: bool,
}

impl InputSpan {
    /// Creates a new InputSpan from a Pest parser Pair.
    ///
    /// # Arguments
    /// * `pair` - Reference to a Pest parser Pair
    pub fn from_pair(pair: &Pair<Rule>) -> Self {
        let (start_line, start_column) = pair.line_col();
        let start = pair.as_span().start();
        let len = pair.as_span().end() - start;
        Self {
            start_line: start_line as u32,
            start_column: start_column as u32,
            start: start as u32,
            len: len as u32,
            tempered: false,
        }
    }

    /// Creates a new InputSpan from a Pest Span.
    ///
    /// # Arguments
    /// * `span` - A Pest Span object
    pub fn from_span(span: Span) -> Self {
        let (start_line, column_start) = span.start_pos().line_col();
        Self {
            start_line: start_line as u32,
            start_column: column_start as u32,
            start: span.start() as u32,
            len: (span.end() - span.start()) as u32,
            tempered: false,
        }
    }

    /// Extracts the text corresponding to this span from the given source text.
    ///
    /// # Arguments
    /// * `text` - The source text to extract from
    ///
    /// # Returns
    /// * `Ok(&str)` - The extracted text slice if the span is valid
    /// * `Err(String)` - Error message if the span is out of bounds
    pub fn span_text<'a>(&self, text: &'a str) -> Result<&'a str, String> {
        let _indices = text.char_indices();
        let end = (self.start + self.len) as usize;
        let start = self.start as usize;
        let len = text.chars().count();
        if start > len || end > len {
            return Err(format!(
                "Span out of bounds: {}..{} (text len: {})",
                start,
                end,
                text.len()
            ));
        }
        Ok(utf8_slice::slice(text, start, end))
    }
}

/// A wrapper type that associates a value with its location in source code.
///
/// # Type Parameters
/// * `T` - The type of value being wrapped, must implement Debug and Serialize
#[derive(Clone, Serialize)]
pub struct Spanned<T>
where
    T: Debug + Serialize,
{
    pub value: T,
    span: InputSpan,
}

impl<T: Debug + Serialize> Spanned<T> {
    /// Creates a new Spanned value.
    ///
    /// # Arguments
    /// * `value` - The value to wrap
    /// * `span` - The source location information
    pub fn new(value: T, span: InputSpan) -> Self {
        Self { value, span }
    }

    /// Returns a reference to the span information.
    pub fn span(&self) -> &InputSpan {
        &self.span
    }

    /// Consumes the Spanned and returns a tuple of the value and span.
    pub fn into_tuple(self) -> (T, InputSpan) {
        (self.value, self.span)
    }

    /// Returns a reference to the wrapped value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the Spanned and returns just the wrapped value.
    pub fn into_span_value(self) -> T {
        self.value
    }

    /// Extracts the text corresponding to this span from the given source text.
    ///
    /// # Arguments
    /// * `text` - The source text to extract from
    pub fn span_text<'a>(&self, text: &'a str) -> Result<&'a str, String> {
        self.span.span_text(text)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const ISpanned: &'static str = r#"
export type SerializedSpanned<T> = {
    value: T,
    span: InputSpan
}
"#;

impl<T: Debug + Serialize> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Debug + Serialize> Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.value))
    }
}

impl<T: Debug + Serialize> Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Represents a compilation error with location information and error details.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize)]
pub struct CompilationError {
    kind: Box<ParseError>,
    span: Box<InputSpan>,
    text: String,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const ICompilationError: &'static str = r#"
export type SerializedCompilationError = {
    kind: ParseError,
    span: InputSpan,
    text: string
}
"#;

impl CompilationError {
    /// Creates a new CompilationError.
    ///
    /// # Arguments
    /// * `kind` - The type of parse error
    /// * `span` - Location information for the error
    /// * `text` - Additional error message text
    pub fn new(kind: ParseError, span: InputSpan, text: String) -> Self {
        Self {
            kind: Box::new(kind),
            span: Box::new(span),
            text,
        }
    }

    /// Creates a CompilationError from a Pest parser Pair.
    ///
    /// # Arguments
    /// * `kind` - The type of parse error
    /// * `pair` - Reference to the Pest parser Pair
    /// * `exclude_string` - Whether to exclude the parsed string from the error message
    pub fn from_pair(kind: ParseError, pair: &Pair<Rule>, exclude_string: bool) -> Self {
        let text = if exclude_string {
            "".to_string()
        } else {
            format!(" {} ({:?})", pair.as_str(), pair.as_rule())
        };
        let span = InputSpan::from_pair(pair);
        Self::new(kind, span, text)
    }

    /// Formats the error message using the original source text.
    ///
    /// # Arguments
    /// * `source` - The original source text
    pub fn to_string_from_source(&self, source: &str) -> String {
        let span_text = self.span.span_text(source);
        let span_text = span_text.unwrap_or("");
        format!(
            "Error at line {}:{} ({})\n\t{}",
            self.span.start_line, self.span.start_column, span_text, self.kind
        )
    }

    /// Returns a formatted error string without source context.
    pub fn to_error_string(&self) -> String {
        format!("{} {}", self.kind, self.text)
    }
}

impl std::fmt::Debug for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "Error at line {}:{}\n\t{}{}",
            self.span.start_line, self.span.start_column, self.kind, self.text
        );
        f.write_str(&s)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl CompilationError {
    pub fn to_error_string_wasm(&self) -> String {
        self.to_error_string()
    }
    pub fn to_string_from_source_wasm(&self, source: &str) -> String {
        self.to_string_from_source(source)
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
    pub fn get_span_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.span).unwrap()
    }
    pub fn get_kind_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.kind).unwrap()
    }
}

/// Represents different types of parsing errors that can occur during compilation.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum ParseError {
    /// Indicates an unexpected token was encountered during parsing
    UnexpectedToken(String),
    /// Indicates a required token was not found during parsing
    MissingToken(String),
    /// Indicates a semantic error in the parsed code
    SemanticError(String),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
pub const IParseError: &'static str = r#"
export type ParseError = {
    type: "UnexpectedToken",
    value: string
} | {
    type: "MissingToken",
    value: string
} | {
    type: "SemanticError",
    value: string
} | {
    type: "WrongNumberOfArguments",
    value: {
        got: number,
        expected: string[],
        name: string
    }
}
"#;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UnexpectedToken(s) => format!("[Unexpected token] {}", s),
            Self::MissingToken(s) => format!("[Missing token] {}", s),
            Self::SemanticError(s) => format!("[Semantic error] {}", s),
        };
        f.write_str(&s)
    }
}

/// Removes multiple elements from a vector by their indices.
///
/// # Arguments
/// * `vec` - The vector to remove elements from
/// * `indices` - Slice containing the indices to remove
pub(crate) fn remove_many<T>(vec: &mut Vec<T>, indices: &[usize]) {
    let mut i = 0; // ugh
    vec.retain(|_| {
        let keep = !indices.contains(&i);
        i += 1;
        keep
    });
}

#[cfg(target_arch = "wasm32")]
pub fn serialize_json_compatible<T>(obj: &T) -> Result<JsValue, serde_wasm_bindgen::Error>
where
    T: Serialize,
{
    let s = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    obj.serialize(&s)
}
