use crate::{math::operators::{BinOp, UnOp}, parser::transformer::TransformError};

use super::{primitive::{Primitive, PrimitiveKind}, primitive_traits::{ApplyOp, OperatorError, Spreadable}};


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
    pub fn to_string(&self) -> String {
        format!(
            "({})",
            self.0
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl ApplyOp for Tuple {
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
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