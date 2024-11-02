use core::fmt;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::parser::il::il_exp::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::VariableKind;
use crate::primitives::PrimitiveKind;
use crate::traits::ToLatex;
use crate::type_checker::type_checker_context::{
    FunctionContext, TypeCheckable, TypeCheckerContext, WithType,
};
use crate::utils::{InputSpan, Spanned};

#[derive(Debug, Serialize, Clone)]
pub struct IterableSet {
    pub var: VariableKind,
    pub iterator: Spanned<PreExp>,
    pub span: InputSpan,
}

#[wasm_bindgen(typescript_custom_section)]
const IIterableSet: &'static str = r#"
export type SerializedIterableSet = {
    var: SerializedVariableKind,
    iterator: SerializedSpanned<SerializedPreExp>,
    span: InputSpan,
}
"#;

impl ToLatex for IterableSet {
    fn to_latex(&self) -> String {
        let var = self.var.to_latex();
        let iterator = self.iterator.to_latex();
        format!("{} \\in {}", var, iterator)
    }
}

impl IterableSet {
    pub fn new(var: VariableKind, iterator: Spanned<PreExp>, span: InputSpan) -> Self {
        Self {
            var,
            iterator,
            span,
        }
    }
    pub fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) {
        self.iterator.populate_token_type_map(context, fn_context);
        let iter_type = self.iterator.get_type(context, fn_context);
        let iter_type = match iter_type {
            PrimitiveKind::Iterable(kind) => *kind,
            _ => PrimitiveKind::Undefined, //should this be undefined or any?
        };
        match &self.var {
            VariableKind::Single(name) => context.add_token_type_or_undefined(
                iter_type,
                self.span.clone(),
                Some(name.value().clone()),
            ),
            VariableKind::Tuple(vars) => match &iter_type {
                PrimitiveKind::Iterable(kind) => {
                    for v in vars {
                        context.add_token_type_or_undefined(
                            *kind.clone(),
                            v.span().clone(),
                            Some(v.value().clone()),
                        )
                    }
                }
                _ => {
                    let types = iter_type.can_spread_into().unwrap_or(Vec::new());
                    for (i, v) in vars.iter().enumerate() {
                        context.add_token_type_or_undefined(
                            types.get(i).unwrap_or(&PrimitiveKind::Undefined).clone(),
                            v.span().clone(),
                            Some(v.value().clone()),
                        )
                    }
                }
            },
        }
    }
    pub fn variable_types(
        &self,
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<Vec<(Spanned<String>, PrimitiveKind)>, TransformError> {
        let iter_type = self.iterator.get_type(context, fn_context);

        let iter_type = match iter_type {
            PrimitiveKind::Iterable(kind) => *kind,
            _ => {
                return Err(TransformError::from_wrong_type(
                    PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
                    iter_type,
                    self.span.clone(),
                ));
            }
        };
        match &self.var {
            VariableKind::Single(name) => Ok(vec![(name.clone(), iter_type)]),
            VariableKind::Tuple(vars) => {
                match &iter_type {
                    PrimitiveKind::Iterable(kind) => {
                        Ok(vars
                            .iter()
                            .map(|v| (v.clone(), *kind.clone()))
                            .collect::<Vec<_>>()) //we don't know at compile time how many variables there are, so we assume all of them exist
                    }
                    _ => {
                        let spreads_into = iter_type.can_spread_into()?;
                        if vars.len() > spreads_into.len() {
                            let err = TransformError::SpreadError {
                                to_spread: iter_type,
                                in_variables: vars
                                    .iter()
                                    .map(|v| v.value().clone())
                                    .collect::<Vec<_>>(),
                            }
                            .add_span(&self.span);
                            Err(err)
                        } else {
                            Ok(vars
                                .iter()
                                .zip(spreads_into.iter())
                                .map(|(v, t)| (v.clone(), t.clone()))
                                .collect::<Vec<_>>())
                        }
                    }
                }
            }
        }
    }
}

impl fmt::Display for IterableSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in {}", self.var, *self.iterator)
    }
}
