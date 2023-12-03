use std::fmt::Debug;

use pest::iterators::Pair;

use crate::{
    bail_wrong_argument_spanned, match_or_bail_spanned,
    parser::{
        parser::{ArrayAccess, CompoundVariable, Rule},
        transformer::{TransformError, TransformerContext},
    },
    utils::{CompilationError, InputSpan, ParseError, Spanned},
    wrong_argument,
};

use super::{graph::Graph, iterable::IterableKind, primitive::Primitive};

#[derive(Debug)]
pub enum Parameter {
    Number(Spanned<f64>),
    String(Spanned<String>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<ArrayAccess>),
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
            Parameter::String(s) => Ok(Primitive::String(s.get_span_value().clone())),
            Parameter::Variable(s) => match context.get_value(s) {
                Some(value) => Ok(value.clone()),
                None => Err(TransformError::MissingVariable(s.get_span_value().clone())),
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

pub trait FunctionCall: Debug {
    fn from_parameters(pars: Vec<Parameter>, rule: &Pair<Rule>) -> Result<Self, CompilationError>
    where
        Self: Sized;
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError>;
    fn to_string(&self) -> String;
}

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
