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


