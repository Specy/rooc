use serde::Serialize;

use super::function_traits::{default_wrong_number_of_arguments, default_wrong_type, RoocFunction};
use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{
    primitives::{IterableKind, Primitive, PrimitiveKind, Tuple},
    type_checker::type_checker_context::{TypeCheckerContext, WithType},
};

#[derive(Debug, Serialize, Clone)]
pub struct EnumerateArray {}

impl RoocFunction for EnumerateArray {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args[..] {
            [ref iterable] => {
                let array = iterable.as_iterator(context, fn_context)?;
                let values = array.to_primitives();
                let mut result = Vec::new();
                for (i, item) in values.into_iter().enumerate() {
                    result.push(Tuple::new(vec![item.clone(), Primitive::Number(i as f64)]));
                }
                Ok(Primitive::Iterable(IterableKind::Tuples(result)))
            }
            _ => Err(default_wrong_number_of_arguments(self)),
        }
    }

    fn type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![(
            "of_iterable".to_string(),
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
        )]
    }

    fn return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        let arg_type = args
            .first()
            .map(|a| a.get_type(context, fn_context))
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

    fn function_name(&self) -> String {
        "enumerate".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        match args[..] {
            [ref iterable] => {
                let arg_type = iterable.get_type(context, fn_context);
                if !matches!(arg_type, PrimitiveKind::Iterable(_)) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                        arg_type,
                        iterable.span().clone(),
                    ));
                }
                Ok(())
            }
            _ => Err(default_wrong_type(args, self, context, fn_context)),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct LenOfIterableFn {}

impl RoocFunction for LenOfIterableFn {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args[..] {
            [ref of_iterable] => {
                let value = of_iterable.as_iterator(context, fn_context)?;
                Ok(Primitive::PositiveInteger(value.len() as u64))
            }
            _ => Err(default_wrong_number_of_arguments(self)),
        }
    }

    fn type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![(
            "of_iterable".to_string(),
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
        )]
    }

    fn return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::PositiveInteger
    }

    fn function_name(&self) -> String {
        "len".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        match args[..] {
            [ref of_iterable] => {
                let arg_type = of_iterable.get_type(context, fn_context);
                if !matches!(arg_type, PrimitiveKind::Iterable(_)) {
                    return Err(TransformError::from_wrong_type(
                        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                        arg_type,
                        of_iterable.span().clone(),
                    ));
                }
                Ok(())
            }
            _ => Err(default_wrong_type(args, self, context, fn_context)),
        }
    }
}
