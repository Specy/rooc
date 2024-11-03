use core::fmt;

use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use crate::prelude::*;

use crate::math::{BinOp, UnOp};
use crate::parser::model_transformer::TransformError;
use crate::traits::ToLatex;

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tuple(pub Vec<Primitive>);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const ITuple: &'static str = r#"
export type SerializedTuple = SerializedPrimitive[]
"#;

impl Tuple {
    pub fn new(v: Vec<Primitive>) -> Self {
        Self(v)
    }
    pub fn get(&self, index: usize) -> Option<&Primitive> {
        self.0.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Primitive> {
        self.0.get_mut(index)
    }
    pub fn into_primitives(self) -> Vec<Primitive> {
        self.0
    }
    pub fn primitives(&self) -> &Vec<Primitive> {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn get_type(&self) -> PrimitiveKind {
        PrimitiveKind::Tuple(self.inner_types())
    }
    pub fn inner_types(&self) -> Vec<PrimitiveKind> {
        self.0.iter().map(|p| p.get_type()).collect::<Vec<_>>()
    }
}

impl ToLatex for Tuple {
    fn to_latex(&self) -> String {
        format!(
            "({})",
            self.0
                .iter()
                .map(|e| e.to_latex())
                .collect::<Vec<_>>()
                .join(",\\")
        )
    }
}

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!(
            "({})",
            self.0
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        f.write_str(&s)
    }
}

impl ApplyOp for Tuple {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(
            op,
            self.get_type(),
        ))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(op, self.get_type()))
    }
    fn can_apply_binary_op(_: BinOp, _: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(_: UnOp) -> bool {
        false
    }
}

impl Spreadable for Tuple {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Ok(self.0)
    }
}
