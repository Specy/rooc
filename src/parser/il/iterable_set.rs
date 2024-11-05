use core::fmt;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::Serialize;

use crate::parser::il::il_exp::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::VariableKind;
use crate::primitives::PrimitiveKind;
use crate::traits::ToLatex;
use crate::type_checker::type_checker_context::{
    FunctionContext, TypeCheckable, TypeCheckerContext, WithType,
};
use crate::utils::{InputSpan, Spanned};

/// Represents an iterable set expression in the intermediate language.
///
/// An iterable set binds a variable or tuple of variables to elements from an iterator expression.
/// For example: `x in 1..10` or `(a,b) in pairs(matrix)`.
#[derive(Debug, Serialize, Clone)]
pub struct IterableSet {
    /// The variable(s) being bound
    pub var: VariableKind,
    /// The iterator expression producing values
    pub iterator: Spanned<PreExp>,
    /// Source code location information
    pub span: InputSpan,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
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
    /// Creates a new IterableSet.
    ///
    /// # Arguments
    /// * `var` - The variable(s) to bind iterator values to
    /// * `iterator` - The iterator expression
    /// * `span` - Source location information
    pub fn new(var: VariableKind, iterator: Spanned<PreExp>, span: InputSpan) -> Self {
        Self {
            var,
            iterator,
            span,
        }
    }

    /// Populates type information for variables in the type checker context.
    ///
    /// # Arguments
    /// * `context` - The type checker context to populate
    /// * `fn_context` - Function context for type checking
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

    /// Gets the types of variables bound by this iterable set.
    ///
    /// # Arguments
    /// * `context` - Type checker context
    /// * `fn_context` - Function context for type checking
    ///
    /// # Returns
    /// * `Ok(Vec<(Spanned<String>, PrimitiveKind)>)` - List of variable names and their types
    /// * `Err(TransformError)` - If type checking fails
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
