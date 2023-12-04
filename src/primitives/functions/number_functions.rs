use pest::iterators::Pair;

use crate::{primitives::{parameter::Parameter, primitive::Primitive, iterable::IterableKind}, parser::{parser::Rule, transformer::{TransformerContext, TransformError}}, utils::{CompilationError, ParseError}, bail_wrong_number_of_arguments};

use super::function_traits::FunctionCall;



#[derive(Debug)]
pub struct NumericRange{
    from: Parameter,
    to: Parameter, 
    to_inclusive: bool
}
impl NumericRange{
    pub fn new(from: Parameter, to: Parameter, to_inclusive: bool) -> Self {
        Self { from, to, to_inclusive }
    }
}

impl FunctionCall for NumericRange{

    fn from_parameters(mut pars: Vec<Parameter>, origin_rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        match pars.len() {
            2 => Ok(Self {
                from: pars.remove(0),
                to: pars.remove(0),
                to_inclusive: false
            }),
            n => bail_wrong_number_of_arguments!(n, origin_rule, ["Number", "Number"]),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let from = self.from.as_integer(context)?;
        let to = self.to.as_integer(context)?;
        let range = if self.to_inclusive {
            (from..=to).map(|i| i as f64).collect()
        } else {
            (from..to).map(|i| i as f64).collect()
        };
        Ok(Primitive::Iterable(IterableKind::Numbers(range)))
    }
    fn to_string(&self) -> String {
        format!("{}..{}", self.from.to_string(), self.to.to_string())
    }
}