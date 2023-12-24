use pest::iterators::Pair;

use crate::{
    bail_wrong_number_of_arguments,
    parser::{
        parser::Rule,
        transformer::{TransformError, TransformerContext}, pre_parsed_problem::PreExp,
    },
    primitives::{
        iterable::IterableKind,
        primitive::{Primitive}, tuple::Tuple,
    },
    utils::{CompilationError, ParseError},
};

use super::function_traits::FunctionCall;

#[derive(Debug)]
pub struct EnumerateArray {
    array: PreExp,
}

impl FunctionCall for EnumerateArray {
    fn from_parameters(
        mut pars: Vec<PreExp>,
        origin_rule: &Pair<Rule>,
    ) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                array: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, origin_rule, "enumerate", ["Array"]),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let array = self.array.as_iterator(context)?;
        let values = array.to_primitives();
        let mut result = Vec::new();
        for (i, item) in values.into_iter().enumerate() {
            result.push(Tuple::new(vec![item.clone(), Primitive::Number(i as f64)]));
        }
        Ok(Primitive::Iterable(IterableKind::Tuple(result)))
    }
    fn to_string(&self) -> String {
        format!("enumerate({})", self.array.to_string())
    }
}


#[derive(Debug)]
pub struct LenOfIterableFn {
    of_iterable: PreExp,
}
impl FunctionCall for LenOfIterableFn {
    fn from_parameters(
        mut pars: Vec<PreExp>,
        rule: &Pair<Rule>,
    ) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                of_iterable: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, rule,"len", ["Iterable"]),
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
