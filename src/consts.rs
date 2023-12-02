use core::panic;
use std::{collections::HashMap, fmt::Debug, ops::Deref, ops::DerefMut};

use crate::{
    bail_wrong_argument, bail_wrong_argument_spanned, match_or_bail, match_or_bail_spanned,
    parser::{CompoundVariable, PreArrayAccess, Rule},
    transformer::{TransformError, TransformerContext},
    wrong_argument,
};
use pest::{Span, iterators::Pair};

#[derive(Debug, Clone)]
pub struct GraphEdge {
    from: String,
    to: String,
    weight: Option<f64>,
}
impl GraphEdge {
    pub fn new(from: String, to: String, weight: Option<f64>) -> Self {
        Self { from, to, weight }
    }
    pub fn to_string(&self) -> String {
        match self.weight {
            Some(w) => format!("{}:{}", self.to, w),
            None => self.to.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    name: String,
    edges: HashMap<String, GraphEdge>,
}
impl GraphNode {
    pub fn new(name: String, edges: Vec<GraphEdge>) -> Self {
        let edges = edges
            .into_iter()
            .map(|edge| (edge.to.clone(), edge))
            .collect::<HashMap<String, GraphEdge>>();
        Self { name, edges }
    }
    pub fn to_string(&self) -> String {
        let edges = self
            .edges
            .iter()
            .map(|(_, edge)| edge.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}: {{{}}}", self.name, edges)
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    vertices: Vec<GraphNode>,
}
impl Graph {
    pub fn new(vertices: Vec<GraphNode>) -> Self {
        Self { vertices }
    }
    pub fn edges(&self) -> Vec<GraphEdge> {
        self.vertices
            .iter()
            .map(|node| {
                node.edges
                    .values()
                    .map(|edge| edge.clone())
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
    }
    pub fn vertices(&self) -> &Vec<GraphNode> {
        &self.vertices
    }
    pub fn neighbour_of(&self, node_name: &str) -> Result<Vec<&GraphEdge>, TransformError> {
        let node = self
            .vertices
            .iter()
            .find(|n: &&GraphNode| n.name == node_name);
        match node {
            Some(node) => Ok(node.edges.values().collect()),
            None => {
                return Err(TransformError::Other(format!(
                    "node {} not found in graph",
                    node_name
                )))
            }
        }
    }

    pub fn to_string(&self) -> String {
        let nodes = self
            .vertices
            .iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        format!("[{}]", nodes)
    }
}

pub trait FunctionCall: Debug {
    fn from_parameters(pars: Vec<Parameter>, span: &Span) -> Result<Self, CompilationError>
    where
        Self: Sized;
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError>;
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct InputSpan {
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub start: usize,
    pub len: usize,
    pub tempered: bool,
}
impl InputSpan {
    pub fn from_pair(pair: Pair<Rule>) -> Self {
        let (start_line, start_col) = pair.line_col();
    }
    pub fn from_span(span: Span) -> Self {
        let (start_line, column_start) = span.start_pos().line_col();
        let (end_line, column_end) = span.end_pos().line_col();
        Self {
            start_line,
            end_line,
            start_column: column_start,
            end_column: column_end,
            start: span.start(),
            len: span.end() - span.start(),
            tempered: false,
        }
    }
    pub fn default() -> Self {
        Self {
            start_line: 0,
            end_line: 0,
            start_column: 0,
            end_column: 0,
            start: 0,
            len: 0,
            tempered: false,
        }
    }
    pub fn from_tempered_span(span: Span) -> Self {
        let mut new_span = Self::from_pair(span);
        new_span.tempered = true;
        new_span
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

#[derive(Debug, Clone)]
pub enum IterableKind {
    Numbers(Vec<f64>),
    Strings(Vec<String>),
    Edges(Vec<GraphEdge>),
    Nodes(Vec<GraphNode>),
    Tuple(Vec<Vec<Primitive>>),
    Iterable(Vec<IterableKind>),
}
impl IterableKind {
    pub fn to_string(&self) -> String {
        match self {
            IterableKind::Numbers(v) => format!("{:?}", v),
            IterableKind::Strings(v) => format!("{:?}", v),
            IterableKind::Edges(v) => format!("{:?}", v),
            IterableKind::Nodes(v) => format!("{:?}", v),
            IterableKind::Tuple(v) => format!("{:?}", v),
            IterableKind::Iterable(v) => {
                let result = v
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", result)
            }
        }
    }
    pub fn len(&self) -> usize {
        match self {
            IterableKind::Numbers(v) => v.len(),
            IterableKind::Strings(v) => v.len(),
            IterableKind::Edges(v) => v.len(),
            IterableKind::Nodes(v) => v.len(),
            IterableKind::Tuple(v) => v.len(),
            IterableKind::Iterable(v) => v.len(),
        }
    }
    pub fn to_primitive_set(self) -> Vec<Primitive> {
        match self {
            IterableKind::Numbers(v) => v.iter().map(|n| Primitive::Number(*n)).collect(),
            IterableKind::Strings(v) => v
                .into_iter()
                .map(|s| Primitive::String((*s).to_string()))
                .collect(),
            IterableKind::Edges(v) => v
                .iter()
                .map(|e| Primitive::GraphEdge(e.to_owned()))
                .collect(),
            IterableKind::Nodes(v) => v
                .into_iter()
                .map(|n| Primitive::GraphNode(n.to_owned()))
                .collect(),
            IterableKind::Tuple(v) => v.into_iter().map(|t| Primitive::Tuple(t)).collect(),
            IterableKind::Iterable(v) => v.into_iter().map(|i| Primitive::Iterable(i)).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Number(f64),
    String(String),
    //TODO instead of making these, make a recursive IterableKind
    Iterable(IterableKind),
    Graph(Graph),
    GraphEdge(GraphEdge),
    GraphNode(GraphNode),
    Tuple(Vec<Primitive>),
    Undefined,
}

impl Primitive {
    pub fn from_constant_value(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Number(n) => Primitive::Number(n),
            ConstantValue::OneDimArray(v) => Primitive::Iterable(IterableKind::Numbers(v)),
            ConstantValue::TwoDimArray(v) => {
                let inner = v
                    .into_iter()
                    .map(|row| IterableKind::Numbers(row))
                    .collect::<Vec<_>>();
                Primitive::Iterable(IterableKind::Iterable(inner))
            }
            ConstantValue::Graph(g) => Primitive::Graph(g),
            ConstantValue::String(s) => Primitive::String(s),
        }
    }
    pub fn as_number(&self) -> Result<f64, TransformError> {
        match_or_bail!("number", Primitive::Number(n) => Ok(*n) ; (self, self))
    }
    pub fn as_integer(&self) -> Result<i64, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("integer", self)
        } else {
            Ok(n as i64)
        }
    }
    pub fn as_usize(&self) -> Result<usize, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("integer", self)
        } else if n < 0.0 {
            bail_wrong_argument!("positive integer", self)
        } else {
            Ok(n as usize)
        }
    }
    pub fn as_graph(&self) -> Result<&Graph, TransformError> {
        match_or_bail!("graph", 
            Primitive::Graph(g) => Ok(g)
          ; (self, self))
    }
    pub fn as_number_array(&self) -> Result<&Vec<f64>, TransformError> {
        match self {
            Primitive::Iterable(IterableKind::Numbers(a)) => Ok(a),
            _ => bail_wrong_argument!("array1d", self),
        }
    }
    pub fn as_number_matrix(&self) -> Result<Vec<&Vec<f64>>, TransformError> {
        match self {
            Primitive::Iterable(IterableKind::Iterable(a)) => a
                .into_iter()
                .map(|row| match row {
                    IterableKind::Numbers(v) => Ok(v),
                    _ => bail_wrong_argument!("array2d", self),
                })
                .collect::<Result<Vec<_>, _>>(),
            _ => bail_wrong_argument!("array2d", self),
        }
    }
    pub fn as_iterator(&self) -> Result<&IterableKind, TransformError> {
        match_or_bail!("iterable", 
            Primitive::Iterable(i) => Ok(i)
        ; (self, self))
    }
    pub fn as_tuple(&self) -> Result<&Vec<Primitive>, TransformError> {
        match_or_bail!("tuple", Primitive::Tuple(t) => Ok(t) ; (self, self))
    }

    pub fn to_string(&self) -> String {
        //TODO improve this
        match self {
            Primitive::Number(n) => n.to_string(),
            Primitive::String(s) => s.to_string(),
            Primitive::Iterable(i) => match i {
                IterableKind::Numbers(v) => format!("{:?}", v),
                IterableKind::Strings(v) => format!("{:?}", v),
                IterableKind::Edges(v) => format!("{:?}", v),
                IterableKind::Nodes(v) => format!("{:?}", v),
                IterableKind::Tuple(v) => format!("{:?}", v),
                IterableKind::Iterable(v) => {
                    let result = v
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("[{}]", result)
                }
            },
            Primitive::Graph(g) => g.to_string(),
            Primitive::GraphEdge(e) => e.to_string(),
            Primitive::GraphNode(n) => n.to_string(),
            Primitive::Tuple(v) => format!("{:?}", v),
            Primitive::Undefined => "undefined".to_string(),
        }
    }
}
#[derive(Debug)]
pub enum Parameter {
    Number(Spanned<f64>),
    String(Spanned<String>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<PreArrayAccess>),
    FunctionCall(Spanned<Box<dyn FunctionCall>>),
}

impl Parameter {
    pub fn as_span(&self) -> &InputSpan {
        match self {
            Parameter::Number(n) => n.get_span(),
            Parameter::String(s) => s.get_span(),
            Parameter::Variable(s) => s.get_span(),
            Parameter::CompoundVariable(c) => c.get_span(),
            Parameter::ArrayAccess(a) => a.get_span(),
            Parameter::FunctionCall(f) => f.get_span(),
        }
    }

    pub fn as_primitive(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self {
            Parameter::Number(n) => Ok(Primitive::Number(**n)),
            Parameter::String(s) => Ok(Primitive::String(s.value.clone())),
            Parameter::Variable(s) => match context.get_value(s) {
                Some(value) => Ok(value.clone()),
                None => Err(TransformError::MissingVariable(s.value.clone())),
            },
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;
                match context.get_value(&name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(TransformError::MissingVariable(name)),
                }
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                Ok(value)
            }
            Parameter::ArrayAccess(a) => {
                let value = context.get_array_access_value(a)?;
                Ok(Primitive::Number(value))
            }
        }
    }
    //TODO make this a macro
    pub fn as_number(&self, context: &TransformerContext) -> Result<f64, TransformError> {
        let value = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        match_or_bail_spanned!("number", Primitive::Number(n) => Ok(n) ; (value, self))
    }
    pub fn as_integer(&self, context: &TransformerContext) -> Result<i64, TransformError> {
        let n = self
            .as_number(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        if n.fract() != 0.0 {
            bail_wrong_argument_spanned!("integer", self)
        } else {
            Ok(n as i64)
        }
    }
    pub fn as_usize(&self, context: &TransformerContext) -> Result<usize, TransformError> {
        let n = self
            .as_number(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        if n.fract() != 0.0 {
            bail_wrong_argument_spanned!("integer", self)
        } else if n < 0.0 {
            bail_wrong_argument_spanned!("positive integer", self)
        } else {
            Ok(n as usize)
        }
    }
    pub fn as_graph(&self, context: &TransformerContext) -> Result<Graph, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_graph().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn as_number_array(
        &self,
        context: &TransformerContext,
    ) -> Result<Vec<f64>, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_number_array().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn as_number_matrix(
        &self,
        context: &TransformerContext,
    ) -> Result<Vec<Vec<f64>>, TransformError> {
        self.as_primitive(context)
            .map(|p| {
                p.as_number_matrix()
                    .map(|v| v.iter().map(|v| (*v).to_owned()).collect::<Vec<_>>())
            })
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn as_iterator(
        &self,
        context: &TransformerContext,
    ) -> Result<IterableKind, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_iterator().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn to_string(&self) -> String {
        match self {
            Parameter::Number(n) => n.to_string(),
            Parameter::String(s) => s.to_string(),
            Parameter::Variable(s) => s.to_string(),
            Parameter::CompoundVariable(c) => c.to_string(),
            Parameter::ArrayAccess(a) => a.to_string(),
            Parameter::FunctionCall(f) => f.to_string(),
        }
    }
}

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
    String(String),
}
impl ConstantValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::OneDimArray(v) => format!("{:?}", v),
            Self::TwoDimArray(v) => {
                let result = v.iter().map(|row| format!("{:?}", row)).collect::<Vec<_>>();
                format!("[\n{}\n]", result.join(",\n"))
            }
            Self::Graph(g) => {
                format!("Graph {{\n{}\n}}", g.to_string())
            }
            Self::String(s) => format!("\"{}\"", s),
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
    span: InputSpan,
    text: String,
}
impl CompilationError {
    pub fn new(kind: ParseError, span: InputSpan, text: String) -> Self {
        Self { kind, span, text }
    }
    pub fn from_span(kind: ParseError, span: &Span, exclude_string: bool) -> Self {
        let text = if exclude_string {
            "".to_string()
        } else {
            span.as_str().to_string()
        };
        let span = InputSpan::from_pair(*span);
        Self::new(kind, span, text)
    }
    
    pub fn to_string(&self) -> String {
        format!(
            "Error at line {}:{} to {}:{}\n\t{} {}",
            self.span.start_line,
            self.span.start_column,
            self.span.end_line,
            self.span.end_column,
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
            Self::UnexpectedToken(s) => format!("Unexpected token: \"{}\"", s),
            Self::MissingToken(s) => format!("Missing token: \"{}\"", s),
            Self::SemanticError(s) => format!("Semantic error: \"{}\"", s),
            Self::WrongNumberOfArguments(got, expected) => format!(
                "Wrong number of arguments: got {}, expected {}: ({})",
                got,
                expected.len(),
                expected.join(", ")
            ),
        }
    }
}
