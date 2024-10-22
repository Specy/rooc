use core::fmt;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use super::primitive::{Primitive, PrimitiveKind};
use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::traits::ToLatex;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::Spanned,
};

#[derive(Debug, Serialize, Clone)]
pub struct Constant {
    pub name: Spanned<String>,
    pub value: PreExp,
}

#[wasm_bindgen(typescript_custom_section)]
const IConstant: &'static str = r#"
export type SerializedConstant = {
    name: string,
    value: SerializedPreExp
}
"#;

impl ToLatex for Constant {
    fn to_latex(&self) -> String {
        format!(
            "{} &= {}",
            self.name.get_span_value(),
            self.value.to_latex()
        )
    }
}

impl Constant {
    pub fn new(name: Spanned<String>, value: PreExp) -> Self {
        Self { name, value }
    }

    pub fn as_primitive(
        &self,
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        self.value.as_primitive(context, fn_context)
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
            self.name.get_span().clone(),
            Some(self.name.get_span_value().clone()),
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
            self.name.get_span().clone(),
            Some(self.name.get_span_value().clone()),
        )
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {} = {}", self.name.get_span_value(), self.value)
    }
}
