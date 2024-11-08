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
            _ => Err(default_wrong_number_of_arguments(self, args, fn_context)),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
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
            _ => Err(default_wrong_number_of_arguments(self, args, fn_context)),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
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

#[derive(Debug, Serialize, Clone)]
pub struct ZipArrays {}

impl RoocFunction for ZipArrays {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args {
            [] => Ok(Primitive::Iterable(IterableKind::Anys(vec![]))),
            iterables => {
                let iterables = iterables
                    .iter()
                    .map(|arg| arg.as_iterator(context, fn_context))
                    .collect::<Result<Vec<_>, _>>()?;
                let len = iterables.len();
                if len == 0 {
                    return Ok(Primitive::Iterable(IterableKind::Anys(vec![])));
                }
                let primitives = iterables
                    .into_iter()
                    .map(|iter| iter.to_primitives())
                    .collect::<Vec<_>>();
                let shortest = primitives.iter().map(|p| p.len()).min().unwrap();
                let mut result = Vec::new();
                for i in 0..shortest {
                    result.push(Tuple::new(
                        primitives.iter().map(|p| p[i].clone()).collect::<Vec<_>>(),
                    ));
                }
                Ok(Primitive::Iterable(IterableKind::Tuples(result)))
            }
        }
    }

    fn type_signature(
        &self,
        args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        let pars = args.len();
        (0..pars)
            .map(|i| {
                (
                    "arg".to_string() + &i.to_string(),
                    PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                )
            })
            .collect()
    }

    fn return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        let tuple_types = args
            .iter()
            .map(|arg| arg.get_type(context, fn_context))
            .collect::<Vec<_>>();
        let all_iterable = tuple_types
            .iter()
            .all(|t| matches!(t, PrimitiveKind::Iterable(_)));
        if !all_iterable {
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any))
        } else {
            let inner_types = tuple_types
                .iter()
                .map(|t| match t {
                    PrimitiveKind::Iterable(inner) => *inner.clone(),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Tuple(inner_types)))
        }
    }

    fn function_name(&self) -> String {
        "zip".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        for arg in args {
            let arg_type = arg.get_type(context, fn_context);
            if !matches!(arg_type, PrimitiveKind::Iterable(_)) {
                return Err(TransformError::from_wrong_type(
                    PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                    arg_type,
                    arg.span().clone(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ArrayDifference {}

/// given two arrays A and B, return the elements in A that are not in B
impl ArrayDifference {
    pub fn new() -> Self {
        Self {}
    }
}
impl RoocFunction for ArrayDifference {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        if args.len() != 2 {
            return Err(default_wrong_number_of_arguments(self, args, fn_context))
        }
        let first = args[0].as_iterator(context, fn_context)?.to_primitives();
        let second = args[1].as_iterator(context, fn_context)?.to_primitives();
        
        let first = first.into_iter().filter(|i| !second.contains(i)).collect();
        Ok(Primitive::Iterable(IterableKind::Anys(first).flatten()))
    }

    fn type_signature(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        let first = args.first()
            .map(|a| a.get_type(context, fn_context))
            .unwrap_or(PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)));
        vec![
            ("from".to_string(), first.clone()),
            ("other".to_string(), first)
        ]
    }

    fn return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        args.first()
            .map(|a| a.get_type(context, fn_context))
            .unwrap_or(PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)))
    }

    fn function_name(&self) -> String {
        "difference".to_string()
    }
}


#[derive(Debug, Serialize, Clone)]
pub struct ArrayUnion;
impl RoocFunction for ArrayUnion {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        if args.len() != 2 {
            return Err(default_wrong_number_of_arguments(self, args, fn_context))
        }
        let mut first = args[0].as_iterator(context, fn_context)?.to_primitives();
        let second = args[1].as_iterator(context, fn_context)?.to_primitives();
        
        let common: Vec<_> = first.iter().filter_map(|i| second.contains(i).then_some(i.clone())).collect();
        first.extend(common);
        Ok(Primitive::Iterable(IterableKind::Anys(first).flatten()))
    }

    fn type_signature(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        let first = args.first()
            .map(|a| a.get_type(context, fn_context))
            .unwrap_or(PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)));
        vec![
            ("first".to_string(), first.clone()),
            ("second".to_string(), first)
        ]
    }

    fn return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        args.first()
            .map(|a| a.get_type(context, fn_context))
            .unwrap_or(PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)))
    }

    fn function_name(&self) -> String {
        "union".to_string()
    }
}

#[derive(Debug, Serialize, Clone)]

pub struct ArrayIntersection;
impl RoocFunction for ArrayIntersection {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        if args.len() != 2 {
            return Err(default_wrong_number_of_arguments(self, args, fn_context))
        }
        let first = args[0].as_iterator(context, fn_context)?.to_primitives();
        let second = args[1].as_iterator(context, fn_context)?.to_primitives();
        
        let result = first.into_iter().filter(|i| second.contains(i)).collect();
        Ok(Primitive::Iterable(IterableKind::Anys(result).flatten()))
    }

    fn type_signature(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        let first = args.first()
            .map(|a| a.get_type(context, fn_context))
            .unwrap_or(PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)));
        vec![
            ("first".to_string(), first.clone()),
            ("second".to_string(), first)
        ]
    }

    fn return_type(
        &self,
        args: &[PreExp],
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        args.first()
            .map(|a| a.get_type(context, fn_context))
            .unwrap_or(PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)))
    }

    fn function_name(&self) -> String {
        "intersection".to_string()
    }
}
