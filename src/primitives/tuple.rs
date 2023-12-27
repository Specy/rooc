use core::fmt;

use crate::{
    math::operators::{BinOp, UnOp},
    parser::transformer::TransformError,
};

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

#[derive(Debug, Clone)]
pub struct Tuple(pub Vec<Primitive>);
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
    pub fn get_primitives(&self) -> &Vec<Primitive> {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(
            op,
            PrimitiveKind::Tuple,
        ))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            PrimitiveKind::Tuple,
        ))
    }
}

impl Spreadable for Tuple {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Ok(self.0)
    }
}
