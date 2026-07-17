use serde::Serialize;

use super::function_traits::{RoocFunction, default_wrong_number_of_arguments, default_wrong_type};
use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{
    primitives::{IterableKind, Primitive, PrimitiveKind},
    type_checker::type_checker_context::{TypeCheckerContext, WithType},
};

#[derive(Debug, Serialize, Clone)]
pub struct NumericRange {}

impl RoocFunction for NumericRange {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args[..] {
            [ref from, ref to, ref to_inclusive] => {
                let from = from.as_integer_cast(context, fn_context)?;
                let to = to.as_integer_cast(context, fn_context)?;
                let to_inclusive = to_inclusive.as_boolean(context, fn_context)?;
                if from >= 0 && to >= 0 {
                    let from = from as usize;
                    let to = to as usize;
                    let range = if to_inclusive {
                        (from..=to).map(|i| i as u64).collect()
                    } else {
                        (from..to).map(|i| i as u64).collect()
                    };
                    return Ok(Primitive::Iterable(IterableKind::PositiveIntegers(range)));
                }
                let range = if to_inclusive {
                    (from..=to).collect()
                } else {
                    (from..to).collect()
                };
                Ok(Primitive::Iterable(IterableKind::Integers(range)))
            }
            _ => Err(default_wrong_number_of_arguments(self, args, fn_context)),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        vec![
            ("from".to_string(), PrimitiveKind::Integer),
            ("to".to_string(), PrimitiveKind::Integer),
            ("to_inclusive".to_string(), PrimitiveKind::Boolean),
        ]
    }

    fn return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        if let [ref from, ref to] = args[..] {
            let from_type = from.get_type(context, fn_context);
            let to_type = to.get_type(context, fn_context);
            //if we know that the numbers are positive, we can return a positive integer range
            if matches!(from_type, PrimitiveKind::PositiveInteger)
                && matches!(to_type, PrimitiveKind::PositiveInteger)
            {
                return PrimitiveKind::Iterable(Box::new(PrimitiveKind::PositiveInteger));
            }
        }
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Integer))
    }

    fn function_name(&self) -> String {
        "range".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        match args[..] {
            [ref from, ref to, ref to_inclusive] => {
                let from_type = from.get_type(context, fn_context);
                let to_type = to.get_type(context, fn_context);
                let to_inclusive_type = to_inclusive.get_type(context, fn_context);
                if !from_type.is_numeric() {
                    //TODO relaxed type checking for numeric ranges
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        from_type,
                        from.span().clone(),
                    ))
                } else if !to_type.is_numeric() {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        to_type,
                        to.span().clone(),
                    ))
                } else if !matches!(to_inclusive_type, PrimitiveKind::Boolean) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Boolean,
                        to_inclusive_type,
                        to_inclusive.span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Err(default_wrong_type(args, self, context, fn_context)),
        }
    }
}
