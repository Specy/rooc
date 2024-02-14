use core::fmt;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::Spanned,
};
use crate::parser::il::il_exp::PreExp;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::TransformerContext;
use crate::traits::latex::ToLatex;

use super::primitive::{Primitive, PrimitiveKind};

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

    pub fn as_primitive(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        self.value.as_primitive(context)
    }
}

impl WithType for Constant {
    fn get_type(&self, context: &TypeCheckerContext) -> PrimitiveKind {
        self.value.get_type(context)
    }
}

impl TypeCheckable for Constant {
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.value.populate_token_type_map(context);
        let value = self.value.get_type(context);
        context.add_token_type_or_undefined(
            value,
            self.name.get_span().clone(),
            Some(self.name.get_span_value().clone()),
        );
    }
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        self.value.type_check(context)?;
        let value = self.value.get_type(context);
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
