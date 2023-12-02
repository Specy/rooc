use std::fmt::Debug;

use pest::{iterators::Pair, Span};

use crate::{
    consts::{CompilationError, FunctionCall, IterableKind, Parameter, ParseError, Primitive},
    parser::Rule,
    transformer::{TransformError, TransformerContext},
};

#[derive(Debug)]
pub struct EdgesOfGraphFn {
    of_graph: Parameter,
}

impl FunctionCall for EdgesOfGraphFn {
    fn from_parameters(pars: Vec<Parameter>, rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        let len = pars.len();
        match pars.into_iter().next() {
            Some(of_graph) => Ok(Self { of_graph }),
            _ => Err(CompilationError::from_pair(
                ParseError::WrongNumberOfArguments(len, vec!["Graph".to_string()]),
                rule,
                true,
            )),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let graph = self.of_graph.as_graph(context)?;
        let edges = graph.edges();
        Ok(Primitive::Iterable(IterableKind::Edges(edges)))
    }
    fn to_string(&self) -> String {
        format!("edges({})", self.of_graph.to_string())
    }
}
#[derive(Debug)]
pub struct LenOfIterableFn {
    of_iterable: Parameter,
}
impl FunctionCall for LenOfIterableFn {
    fn from_parameters(pars: Vec<Parameter>, rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        let len = pars.len();
        match pars.into_iter().next() {
            Some(of_iterable) => Ok(Self { of_iterable }),
            _ => Err(CompilationError::from_pair(
                ParseError::WrongNumberOfArguments(len, vec!["Iterable".to_string()]),
                rule,
                true,
            )),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let value = self.of_iterable.as_iterator(context)?;
        Ok(Primitive::Number(value.len() as f64))
    }
    fn to_string(&self) -> String {
        format!("len({})", self.of_iterable.to_string())
    }
}

/* guards */

pub trait ToNum: Debug {
    fn to_num(&self, context: &TransformerContext) -> Result<f64, TransformError>;
    fn to_int(&self, context: &TransformerContext) -> Result<i64, TransformError> {
        let num = self.to_num(context)?;
        if num.fract() != 0.0 {
            return Err(TransformError::WrongArgument("Integer".to_string()));
        }
        Ok(num as i64)
    }
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

#[derive(Debug)]
pub struct ParameterToNum {
    parameter: Parameter,
}
impl ParameterToNum {
    pub fn from_parameter(parameter: Parameter) -> Self {
        Self { parameter }
    }
}
impl ToNum for ParameterToNum {
    fn to_num(&self, context: &TransformerContext) -> Result<f64, TransformError> {
        let value = self.parameter.as_number(context)?;
        Ok(value)
    }
}
