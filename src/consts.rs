use std::fmt::format;

use crate::transformer::Graph;
use pest::Span;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Comparison {
    LowerOrEqual,
    UpperOrEqual,
    Equal,
}
impl Comparison {
    pub fn to_string(&self) -> String {
        match self {
            Comparison::LowerOrEqual => "<=".to_string(),
            Comparison::UpperOrEqual => ">=".to_string(),
            Comparison::Equal => "=".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
impl Op {
    pub fn precedence(&self) -> u8 {
        match self {
            Op::Add => 1,
            Op::Sub => 1,
            Op::Mul => 2,
            Op::Div => 2,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Op::Add => "+".to_string(),
            Op::Sub => "-".to_string(),
            Op::Mul => "*".to_string(),
            Op::Div => "/".to_string(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum OptimizationType {
    Min,
    Max,
}
impl OptimizationType {
    pub fn to_string(&self) -> String {
        match self {
            OptimizationType::Min => "min".to_string(),
            OptimizationType::Max => "max".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Number(f64),
    OneDimArray(Vec<f64>),
    TwoDimArray(Vec<Vec<f64>>),
    Graph(Graph),
}
impl ConstantValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::OneDimArray(v) => format!("{:?}", v),
            Self::TwoDimArray(v) => {
                let result = v
                    .iter()
                    .map(|row| format!("{:?}", row))
                    .collect::<Vec<_>>();
                format!("[\n{}\n]", result.join(",\n"))
            }
            Self::Graph(g) => {
                format!("Graph {{\n{}\n}}", g.to_string())
            }
        }
    }
}

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub value: ConstantValue,
}
impl Constant {
    pub fn new(name: String, value: ConstantValue) -> Self {
        Self { name, value }
    }
    pub fn to_string(&self) -> String {
        format!("{} = {}", self.name, self.value.to_string())
    }
}

pub struct CompilationError {
    kind: ParseError,
    start_line: usize,
    start: usize,
    end_line: usize,
    end: usize,
    text: String,
}
impl CompilationError {
    pub fn new(
        kind: ParseError,
        start_line: usize,
        start: usize,
        end_line: usize,
        end: usize,
        text: String,
    ) -> Self {
        Self {
            kind,
            start_line,
            start,
            end_line,
            end,
            text,
        }
    }
    pub fn from_span(kind: ParseError, span: &Span, exclude_string: bool) -> Self {
        let (start_line, start) = span.start_pos().line_col();
        let (end_line, end) = span.end_pos().line_col();
        let text = if exclude_string { "" } else { span.as_str() }.to_string();
        Self::new(kind, start_line, start, end_line, end, text)
    }
    pub fn to_string(&self) -> String {
        format!(
            "Error at line {}:{} to {}:{}\n\t{} {}",
            self.start_line,
            self.start,
            self.end_line,
            self.end,
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
}
impl ParseError {
    pub fn to_string(&self) -> String {
        match self {
            Self::UnexpectedToken(s) => format!("Unexpected token: \"{}\"", s),
            Self::MissingToken(s) => format!("Missing token: \"{}\"", s),
            Self::SemanticError(s) => format!("Semantic error: \"{}\"", s),
        }
    }
}
