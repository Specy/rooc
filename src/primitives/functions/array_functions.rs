use pest::iterators::Pair;
use serde::Serialize;

use crate::{
    bail_wrong_number_of_arguments,
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformError, TransformerContext, TypeCheckerContext},
    },
    primitives::{
        iterable::IterableKind,
        primitive::{Primitive, PrimitiveKind},
        tuple::Tuple,
    },
    utils::{CompilationError, ParseError},
};

use super::function_traits::FunctionCall;

#[derive(Debug, Serialize, Clone)]
pub struct EnumerateArray {
    iterable: PreExp,
}

impl FunctionCall for EnumerateArray {
    fn from_parameters(
        mut pars: Vec<PreExp>,
        origin_rule: &Pair<Rule>,
    ) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                iterable: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, origin_rule, "enumerate", ["Array"]),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let array = self.iterable.as_iterator(context)?;
        let values = array.to_primitives();
        let mut result = Vec::new();
        for (i, item) in values.into_iter().enumerate() {
            result.push(Tuple::new(vec![item.clone(), Primitive::Number(i as f64)]));
        }
        Ok(Primitive::Iterable(IterableKind::Tuple(result)))
    }
    fn to_string(&self) -> String {
        format!("enumerate({})", self.iterable)
    }
    fn type_check(&self, context: &TypeCheckerContext) -> Result<(), TransformError> {
        let arg_type = self.iterable.get_type(context);
        if !matches!(arg_type, PrimitiveKind::Iterable(_)) {
            return Err(TransformError::from_wrong_type(
                PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                arg_type,
                self.iterable.get_span().clone(),
            ));
        }
        Ok(())
    }
    fn get_parameters_types(&self) -> Vec<PrimitiveKind> {
        vec![PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any))]
    }
    fn get_return_type(&self, context: &TypeCheckerContext) -> PrimitiveKind {
        let arg_type = self.iterable.get_type(context);
        let arg_type = match arg_type {
            PrimitiveKind::Iterable(t) => *t,
            _ => PrimitiveKind::Undefined
        };
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Tuple(vec![
            arg_type,
            PrimitiveKind::Number,
        ])))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct LenOfIterableFn {
    of_iterable: PreExp,
}
impl FunctionCall for LenOfIterableFn {
    fn from_parameters(mut pars: Vec<PreExp>, rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                of_iterable: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, rule, "len", ["Iterable"]),
        }
    }

    fn type_check(&self, context: &TypeCheckerContext) -> Result<(), TransformError> {
        let arg_type = self.of_iterable.get_type(context);
        if !matches!(arg_type, PrimitiveKind::Iterable(_)) {
            return Err(TransformError::from_wrong_type(
                PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                arg_type,
                self.of_iterable.get_span().clone(),
            ));
        }
        Ok(())
    }
    fn get_parameters_types(&self) -> Vec<PrimitiveKind> {
        vec![PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any))]
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let value = self.of_iterable.as_iterator(context)?;
        Ok(Primitive::Number(value.len() as f64))
    }
    fn to_string(&self) -> String {
        format!("len({})", self.of_iterable)
    }
    fn get_return_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Number
    }
}
