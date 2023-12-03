use std::fmt::Debug;

use pest::iterators::Pair;

use crate::{parser::{transformer::{TransformerContext, TransformError}, parser::Rule}, primitives::{parameter::Parameter, primitive::Primitive}, utils::CompilationError};


pub trait FunctionCall: Debug {
    fn from_parameters(pars: Vec<Parameter>, origin_rule: &Pair<Rule>) -> Result<Self, CompilationError>
    where
        Self: Sized;
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError>;
    fn to_string(&self) -> String;
}

pub trait ToNum: Debug {
    fn to_num(&self, context: &TransformerContext) -> Result<f64, TransformError>;
    fn to_int(&self, context: &TransformerContext) -> Result<i64, TransformError> {
        let num = self.to_num(context)?;
        if num.fract() != 0.0 {
            return Err(TransformError::WrongArgument(format!(
                "Expected integer, got {}",
                num
            )));
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
            _ => Err(TransformError::WrongArgument(format!(
                "Expected number, got {}",
                value.get_argument_name()
            ))),
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