use core::fmt;
use std::{fmt::Debug, ops::Deref, ops::DerefMut};

use pest::{iterators::Pair, Span};
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::parser::pre_model::Rule;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[wasm_bindgen]
pub struct InputSpan {
    //as u32 as realistically we won't have more than 4 billion characters in a file
    pub start_line: u32,
    pub start_column: u32,
    pub start: u32,
    pub len: u32,
    pub tempered: bool,
}

impl InputSpan {
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

    pub fn get_span_text<'a>(&self, text: &'a str) -> Result<&'a str, String> {
        let end = (self.start + self.len) as usize;
        let start = self.start as usize;
        if start > text.len() || end > text.len() {
            return Err(format!(
                "Span out of bounds: {}..{} (text len: {})",
                start,
                end,
                text.len()
            ));
        }
        Ok(&text[start..end])
    }
}

#[derive(Clone, Serialize)]
pub struct Spanned<T>
where
    T: Debug + Serialize,
{
    pub value: T,
    span: InputSpan,
}

impl<T: Debug + Serialize> Spanned<T> {
    pub fn new(value: T, span: InputSpan) -> Self {
        Self { value, span }
    }
    pub fn get_span(&self) -> &InputSpan {
        &self.span
    }

    pub fn into_tuple(self) -> (T, InputSpan) {
        (self.value, self.span)
    }
    pub fn get_span_value(&self) -> &T {
        &self.value
    }
    pub fn into_span_value(self) -> T {
        self.value
    }
    pub fn get_span_text<'a>(&self, text: &'a str) -> Result<&'a str, String> {
        self.span.get_span_text(text)
    }
}

#[wasm_bindgen(typescript_custom_section)]
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

#[wasm_bindgen]
#[derive(Serialize)]
pub struct CompilationError {
    kind: Box<ParseError>,
    span: Box<InputSpan>,
    text: String,
}

#[wasm_bindgen(typescript_custom_section)]
pub const ICompilationError: &'static str = r#"
export type SerializedCompilationError = {
    kind: ParseError,
    span: InputSpan,
    text: string
}
"#;

impl CompilationError {
    pub fn new(kind: ParseError, span: InputSpan, text: String) -> Self {
        Self {
            kind: Box::new(kind),
            span: Box::new(span),
            text,
        }
    }
    pub fn from_pair(kind: ParseError, pair: &Pair<Rule>, exclude_string: bool) -> Self {
        let text = if exclude_string {
            "".to_string()
        } else {
            format!(" {} ({:?})", pair.as_str(), pair.as_rule())
        };
        let span = InputSpan::from_pair(pair);
        Self::new(kind, span, text)
    }

    pub fn to_string_from_source(&self, source: &str) -> String {
        let span_text = self.span.get_span_text(source);
        let span_text = span_text.unwrap_or("");
        format!(
            "Error at line {}:{} ({})\n\t{}",
            self.span.start_line, self.span.start_column, span_text, self.kind
        )
    }
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

#[wasm_bindgen]
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

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum ParseError {
    UnexpectedToken(String),
    MissingToken(String),
    SemanticError(String),
}

#[wasm_bindgen(typescript_custom_section)]
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

pub fn remove_many<T>(vec: &mut Vec<T>, indices: &[usize]) {
    let mut i = 0; // ugh
    vec.retain(|_| {
        let keep = !indices.contains(&i);
        i += 1;
        keep
    });
}
