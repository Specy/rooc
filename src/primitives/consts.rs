use core::fmt;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::Serialize;

use super::primitive::{Primitive, PrimitiveKind};
use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::traits::ToLatex;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::utils::InputSpan;
use crate::{
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::Spanned,
};

#[derive(Debug, Serialize, Clone)]
/// A constant value with a name and a value
pub struct Constant {
    pub name: Spanned<String>,
    pub value: PreExp,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IConstant: &'static str = r#"
export type SerializedConstant = {
    name: string,
    value: SerializedPreExp
}
"#;

impl ToLatex for Constant {
    fn to_latex(&self) -> String {
        format!("{} &= {}", self.name.value(), self.value.to_latex())
    }
}

impl Constant {
    pub(crate) fn new(name: Spanned<String>, value: PreExp) -> Self {
        Self { name, value }
    }

    /// Get the primitive value of the constant, it evaluates the value of the expression
    pub fn as_primitive(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        self.value.as_primitive(context, fn_context)
    }

    /// Create a new constant from a primitive value
    pub fn from_primitive(name: &str, primitive: Primitive) -> Self {
        let primitive = Spanned::new(primitive, InputSpan::default());
        Self::new(
            Spanned::new(name.to_string(), InputSpan::default()),
            PreExp::Primitive(primitive),
        )
    }
}

impl WithType for Constant {
    fn get_type(
        &self,
        context: &TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        self.value.get_type(context, fn_context)
    }
}

impl TypeCheckable for Constant {
    fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) {
        self.value.populate_token_type_map(context, fn_context);
        let value = self.value.get_type(context, fn_context);
        context.add_token_type_or_undefined(
            value,
            self.name.span().clone(),
            Some(self.name.value().clone()),
        );
    }
    fn type_check(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        self.value.type_check(context, fn_context)?;
        let value = self.value.get_type(context, fn_context);
        context.add_token_type(
            value,
            self.name.span().clone(),
            Some(self.name.value().clone()),
        )
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {} = {}", self.name.value(), self.value)
    }
}
