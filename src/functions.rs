use std::{fmt::Debug};

use pest::Span;

use crate::{
    consts::{CompilationError, FunctionCall, IterableKind, Parameter, ParseError, Primitive},
    transformer::{TransformError, TransformerContext},
};

#[derive(Debug)]
pub struct EdgesOfGraphFn {
    of_graph: Parameter,
}

impl FunctionCall for EdgesOfGraphFn {
    fn from_parameters(pars: Vec<Parameter>, span: &Span) -> Result<Self, CompilationError> {
        let len = pars.len();
        match pars.into_iter().next() {
            Some(of_graph) => Ok(Self { of_graph }),
            _ => Err(CompilationError::from_span(
                ParseError::WrongNumberOfArguments(len, vec!["Graph".to_string()]),
                span,
                true,
            )),
        }
    }
    fn call<'a>(&self, context: &'a TransformerContext) -> Result<Primitive<'a>, TransformError> {
        let graph = self.of_graph.as_graph(context)?;
        let edges = graph.edges();
        Ok(Primitive::Iterable(IterableKind::Edges(edges)))
    }
    fn to_string(&self) -> String {
        format!("edges({})", self.of_graph.to_string())
    }
}

/* guards */

pub trait ToNum: Debug {
    fn to_num(&self, context: &TransformerContext) -> Result<f64, TransformError>;
}

#[derive(Debug)]
pub struct FunctionCallNumberGuard {
    function: Box<dyn FunctionCall>,
}
impl FunctionCallNumberGuard {
    pub fn new(function: Box<dyn FunctionCall>) -> Self {
        Self { function }
    }
}
//TODO make this a macro
impl ToNum for FunctionCallNumberGuard {
    fn to_num(&self, context: &TransformerContext) -> Result<f64, TransformError> {
        let value = self.function.call(context)?;
        match value {
            Primitive::Number(n) => Ok(n),
            _ => Err(TransformError::WrongArgument("Number".to_string())),
        }
    }
}
#[derive(Debug)]
pub struct StaticNumberGuard {
    value: f64,
}
impl StaticNumberGuard {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}
impl ToNum for StaticNumberGuard {
    fn to_num(&self, _context: &TransformerContext) -> Result<f64, TransformError> {
        Ok(self.value)
    }
}
