use crate::{
    bail_wrong_number_of_arguments,
    parser::{
        parser::Rule,
        transformer::{TransformError, TransformerContext},
    },
    primitives::{parameter::Parameter, primitive::Primitive},
    utils::{CompilationError, ParseError},
};
use pest::iterators::Pair;
use std::fmt::Debug;

use super::function_traits::FunctionCall;

#[derive(Debug)]
pub struct LenOfIterableFn {
    of_iterable: Parameter,
}
impl FunctionCall for LenOfIterableFn {
    fn from_parameters(
        mut pars: Vec<Parameter>,
        rule: &Pair<Rule>,
    ) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                of_iterable: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, rule, ["Iterable"]),
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
