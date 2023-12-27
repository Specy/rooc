use core::fmt;
use std::{fmt::Debug, ops::Deref, ops::DerefMut};

use pest::{iterators::Pair, Span};

use crate::parser::parser::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct InputSpan {
    pub start_line: usize,
    pub start_column: usize,
    pub start: usize,
    pub len: usize,
    pub tempered: bool,
}
impl InputSpan {
    pub fn from_pair(pair: &Pair<Rule>) -> Self {
        let (start_line, start_column) = pair.line_col();
        let start = pair.as_span().start();
        let len = pair.as_span().end() - start;
        Self {
            start_line,
            start_column,
            start,
            len,
            tempered: false,
        }
    }
    pub fn from_span(span: Span) -> Self {
        let (start_line, column_start) = span.start_pos().line_col();
        Self {
            start_line,
            start_column: column_start,
            start: span.start(),
            len: span.end() - span.start(),
            tempered: false,
        }
    }

    pub fn get_span_text<'a>(&self, text: &'a str) -> Result<&'a str, ()> {
        let start = self.start;
        let end = start + self.len;
        if start >= text.len() || end >= text.len() {
            return Err(());
        }
        Ok(&text[start..end])
    }
}


#[derive(Clone)]
pub struct Spanned<T>
where
    T: Debug,
{
    pub value: T,
    span: InputSpan,
}
impl<T: Debug> Spanned<T> {
    pub fn new(value: T, span: InputSpan) -> Self {
        Self { value, span }
    }
    pub fn get_span(&self) -> &InputSpan {
        &self.span
    }

    pub fn get_span_value(&self) -> &T {
        &self.value
    }
    pub fn into_span_value(self) -> T {
        self.value
    }
    pub fn get_span_text<'a>(&self, text: &'a str) -> Result<&'a str, ()> {
        self.span.get_span_text(text)
    }
}

impl<T: Debug> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.value))
    }
}
impl<T: Debug> Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct CompilationError {
    kind: Box<ParseError>,
    span: Box<InputSpan>,
    text: String,
}
impl CompilationError {
    pub fn new(kind: ParseError, span: InputSpan, text: String) -> Self {
        Self { 
            kind: Box::new(kind),
            span: Box::new(span),
            text
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

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    MissingToken(String),
    SemanticError(String),
    WrongNumberOfArguments {
        got: usize,
        expected: Vec<String>,
        name: String,
    },
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UnexpectedToken(s) => format!("[Unexpected token] {}", s),
            Self::MissingToken(s) => format!("[Missing token] {}", s),
            Self::SemanticError(s) => format!("[Semantic error] {}", s),
            Self::WrongNumberOfArguments {
                got,
                expected,
                name,
            } => format!(
                "[Wrong number of arguments] got {}, expected {}, for \"function {}({})\"",
                got,
                expected.len(),
                name,
                expected
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("p{}: {}", i, s))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        };
        f.write_str(&s)
    }
}
