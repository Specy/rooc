use pest::iterators::Pair;
use serde::Serialize;

use crate::parser::il::il_exp::PreExp;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::TransformerContext;
use crate::{
    bail_incorrect_type_signature, bail_incorrect_type_signature_of_fn,
    bail_wrong_number_of_arguments,
    parser::parser::Rule,
    primitives::{
        iterable::IterableKind,
        primitive::{Primitive, PrimitiveKind},
        tuple::Tuple,
    },
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::InputSpan,
};

use super::function_traits::FunctionCall;

#[derive(Debug, Serialize, Clone)]
pub struct EnumerateArray {
    args: Vec<PreExp>,
    span: InputSpan,
}

impl TypeCheckable for EnumerateArray {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self.args[..] {
            [ref iterable] => {
                let arg_type = iterable.get_type(context);
                if !matches!(arg_type, PrimitiveKind::Iterable(_)) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                        arg_type,
                        iterable.get_span().clone(),
                    ));
                }
                Ok(())
            }
            _ => bail_incorrect_type_signature_of_fn!(self, context),
        }
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.args
            .iter()
            .for_each(|arg| arg.populate_token_type_map(context));
    }
}

impl WithType for EnumerateArray {
    fn get_type(&self, context: &TypeCheckerContext) -> PrimitiveKind {
        let arg_type = self
            .args
            .get(0)
            .map(|a| a.get_type(context))
            .unwrap_or(PrimitiveKind::Undefined);
        let arg_type = match arg_type {
            PrimitiveKind::Iterable(t) => *t,
            _ => PrimitiveKind::Undefined,
        };
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Tuple(vec![
            arg_type,
            PrimitiveKind::PositiveInteger,
        ])))
    }
}

impl FunctionCall for EnumerateArray {
    fn from_parameters(args: Vec<PreExp>, rule: &Pair<Rule>) -> Self {
        Self {
            args,
            span: InputSpan::from_pair(rule),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self.args[..] {
            [ref iterable] => {
                let array = iterable.as_iterator(context)?;
                let values = array.to_primitives();
                let mut result = Vec::new();
                for (i, item) in values.into_iter().enumerate() {
                    result.push(Tuple::new(vec![item.clone(), Primitive::Number(i as f64)]));
                }
                Ok(Primitive::Iterable(IterableKind::Tuple(result)))
            }
            _ => bail_wrong_number_of_arguments!(self),
        }
    }
    fn to_string(&self) -> String {
        format!(
            "{}({})",
            self.get_function_name(),
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
    fn get_function_name(&self) -> String {
        "enumerate".to_string()
    }

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![(
            "of_iterable".to_string(),
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
        )]
    }
    fn get_parameters(&self) -> &Vec<PreExp> {
        &self.args
    }
    fn get_span(&self) -> &InputSpan {
        &self.span
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct LenOfIterableFn {
    args: Vec<PreExp>,
    span: InputSpan,
}

impl TypeCheckable for LenOfIterableFn {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self.args[..] {
            [ref of_iterable] => {
                let arg_type = of_iterable.get_type(context);
                if !matches!(arg_type, PrimitiveKind::Iterable(_)) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                        arg_type,
                        of_iterable.get_span().clone(),
                    ));
                }
                Ok(())
            }
            _ => bail_incorrect_type_signature_of_fn!(self, context),
        }
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.args
            .iter()
            .for_each(|arg| arg.populate_token_type_map(context));
    }
}

impl WithType for LenOfIterableFn {
    fn get_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::PositiveInteger
    }
}

impl FunctionCall for LenOfIterableFn {
    fn from_parameters(args: Vec<PreExp>, rule: &Pair<Rule>) -> Self {
        Self {
            args,
            span: InputSpan::from_pair(rule),
        }
    }
    fn get_span(&self) -> &InputSpan {
        &self.span
    }
    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![(
            "of_iterable".to_string(),
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
        )]
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self.args[..] {
            [ref of_iterable] => {
                let value = of_iterable.as_iterator(context)?;
                Ok(Primitive::Number(value.len() as f64))
            }
            _ => bail_wrong_number_of_arguments!(self),
        }
    }
    fn to_string(&self) -> String {
        format!(
            "{}({})",
            self.get_function_name(),
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
    fn get_function_name(&self) -> String {
        "len".to_string()
    }
    fn get_parameters(&self) -> &Vec<PreExp> {
        &self.args
    }
}
