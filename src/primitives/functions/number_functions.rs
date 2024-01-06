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
                if !matches!(from_type, PrimitiveKind::Number) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Number,
                        from_type,
                        from.get_span().clone(),
                    ))
                } else if !matches!(to_type, PrimitiveKind::Number) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Number,
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
    fn get_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Number))
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
                let from = from.as_integer(context)?;
                let to = to.as_integer(context)?;
                let to_inclusive = to_inclusive.as_boolean(context)?;
                let range = if to_inclusive {
                    (from..=to).map(|i| i as f64).collect()
                } else {
                    (from..to).map(|i| i as f64).collect()
                };
                Ok(Primitive::Iterable(IterableKind::Numbers(range)))
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
        vec![PrimitiveKind::Number; 2]
    }
}
