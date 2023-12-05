use std::{fmt::Debug, ops::Deref, ops::DerefMut};

use pest::{iterators::Pair, Span};

use crate::parser::parser::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn default() -> Self {
        Self {
            start_line: 0,
            start_column: 0,
            start: 0,
            len: 0,
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
    value: T,
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
    kind: ParseError,
    span: InputSpan,
    text: String,
}
impl CompilationError {
    pub fn new(kind: ParseError, span: InputSpan, text: String) -> Self {
        Self { kind, span, text }
    }
    pub fn from_pair(kind: ParseError, pair: &Pair<Rule>, exclude_string: bool) -> Self {
        let text = if exclude_string {
            "".to_string()
        } else {
            format!(" {} ({:?})", pair.as_str().to_string(), pair.as_rule())
        };
        let span = InputSpan::from_pair(pair);
        Self::new(kind, span, text)
    }

    pub fn to_string(&self) -> String {
        format!(
            "Error at line {}:{}\n\t{}{}",
            self.span.start_line,
            self.span.start_column,
            self.kind.to_string(),
            self.text
        )
    }
    pub fn to_error_string(&self) -> String {
        format!("{} {}", self.kind.to_string(), self.text)
    }
}
impl std::fmt::Debug for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    MissingToken(String),
    SemanticError(String),
    WrongNumberOfArguments(usize, Vec<String>),
}
impl ParseError {
    pub fn to_string(&self) -> String {
        match self {
            Self::UnexpectedToken(s) => format!("[Unexpected token] {}", s),
            Self::MissingToken(s) => format!("[Missing token]\n\t{}", s),
            Self::SemanticError(s) => format!("[Semantic error]\n\t{}", s),
            Self::WrongNumberOfArguments(got, expected) => format!(
                "[Wrong number of arguments]\n\tgot {}, expected {}: ({})",
                got,
                expected.len(),
                expected.join(", ")
            ),
        }
    }
}
