use pest::{iterators::Pair, Span};
use serde::Serialize;

use crate::{
    bail_wrong_number_of_arguments,
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformError, TransformerContext},
    },
    primitives::{
        iterable::IterableKind,
        primitive::{Primitive, PrimitiveKind},
    },
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::{CompilationError, InputSpan, ParseError, Spanned},
    wrong_argument, bail_incorrect_type_signature, bail_incorrect_type_signature_of_fn,
};

use super::function_traits::FunctionCall;

#[derive(Debug, Serialize, Clone)]
pub struct NumericRange {
    args: Vec<PreExp>,
    span: InputSpan,
    known_inclusive: Option<bool>,
}
impl NumericRange {
    pub fn new(from: PreExp, to: PreExp, to_inclusive: bool, span: Span) -> Self {
        Self {
            args: vec![
                from,
                to,
                PreExp::Primitive(Spanned::new(
                    Primitive::Boolean(to_inclusive),
                    InputSpan::default(),
                )),
            ],
            known_inclusive: Some(to_inclusive),
            span: InputSpan::from_span(span),
        }
    }
}

impl TypeCheckable for NumericRange {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self.args[..]{
            [ref from, ref to, ref to_inclusive] => {
                let from_type = from.get_type(context);
                let to_type = to.get_type(context);
                let to_inclusive_type = to_inclusive.get_type(context);
                if !from_type.is_numeric() { //TODO relaxed type checking for numeric ranges
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        from_type,
                        from.get_span().clone(),
                    ))
                } else if !to_type.is_numeric() {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Integer,
                        to_type,
                        to.get_span().clone(),
                    ))
                } else if !matches!(to_inclusive_type, PrimitiveKind::Boolean) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Boolean,
                        to_inclusive_type,
                        to_inclusive.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => bail_incorrect_type_signature_of_fn!(self, context)
        }
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.args
        .iter()
        .for_each(|arg| arg.populate_token_type_map(context));

    }
}
impl WithType for NumericRange {
    fn get_type(&self, context: &TypeCheckerContext) -> PrimitiveKind {
        match self.args[..] {
            [ref from, ref to] => {
                let from_type = from.get_type(context);
                let to_type = to.get_type(context);
                //if we know that the numbers are positive, we can return a positive integer range
                if matches!(from_type, PrimitiveKind::PositiveInteger) && matches!(to_type, PrimitiveKind::PositiveInteger) {
                    return PrimitiveKind::Iterable(Box::new(PrimitiveKind::PositiveInteger));
                }
            }
            _ => {}
        }
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Integer))
    }
}
impl FunctionCall for NumericRange {
    fn from_parameters(args: Vec<PreExp>, rule: &Pair<Rule>) -> Self {
        Self { args, span: InputSpan::from_pair(rule), known_inclusive: None }
    }
    fn get_parameters(&self) -> &Vec<PreExp> {
        &self.args
    }
    fn get_span(&self) -> &InputSpan {
        &self.span
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self.args[..] {
            [ref from,ref  to, ref to_inclusive] => {
                let from = from.as_integer_cast(context)?;
                let to = to.as_integer_cast(context)?;
                let to_inclusive = to_inclusive.as_boolean(context)?;
                if from >= 0 && to >=0 {
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
                    (from..=to).map(|i| i as i64).collect()
                } else {
                    (from..to).map(|i| i as i64).collect()
                };
                Ok(Primitive::Iterable(IterableKind::Integers(range)))
            }
            _ => bail_wrong_number_of_arguments!(self),
        }
        
    }
    fn to_string(&self) -> String {
        match self.args[..] {
            [ref from, ref to, _] => {
                if let Some(inclusive) = self.known_inclusive {
                    let range = if inclusive { "..=" } else { ".." };
                    return format!("{}{}{}", from.to_string(), range, to.to_string());
                }
            } 
            _ => {}
        }
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
        "range".to_string()
    }
    fn get_type_signature(&self) -> Vec<PrimitiveKind> {
        vec![PrimitiveKind::Integer, PrimitiveKind::Integer, PrimitiveKind::Boolean]
    }
}
