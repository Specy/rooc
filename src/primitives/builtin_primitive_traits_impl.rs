use crate::{math::operators::{BinOp, UnOp}, parser::transformer::TransformError};

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

/* --------- ApplyOp --------- */
impl ApplyOp for f64 {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match to {
            Primitive::Number(n) => match op {
                BinOp::Add => Ok(Primitive::Number(self + n)),
                BinOp::Sub => Ok(Primitive::Number(self - n)),
                BinOp::Mul => Ok(Primitive::Number(self * n)),
                BinOp::Div => Ok(Primitive::Number(self / n)),
            },
            _ => Err(OperatorError::incompatible_type(
                op,
                PrimitiveKind::Number,
                to.get_type(),
            )),
        }
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        match op {
            UnOp::Neg => Ok(Primitive::Number(-self)),
        }
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        matches!(to, PrimitiveKind::Number)
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        matches!(op, UnOp::Neg)
    }
}
impl ApplyOp for String {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match to {
            Primitive::String(s) => match op {
                BinOp::Add => Ok(Primitive::String(format!("{}{}", self, s))),
                _ => Err(OperatorError::unsupported_bin_operation(
                    op,
                    PrimitiveKind::String,
                )),
            },
            _ => Err(OperatorError::incompatible_type(
                op,
                PrimitiveKind::String,
                to.get_type(),
            )),
        }
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            PrimitiveKind::String,
        ))
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        matches!(to, PrimitiveKind::String)
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        false
    }
}
impl ApplyOp for bool {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(
            op,
            PrimitiveKind::Boolean,
        ))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            PrimitiveKind::Boolean,
        ))
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        false
    }
}




/* --------- Spreadable --------- */

impl Spreadable for f64 {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::Number))
    }
}
impl Spreadable for String {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::String))
    }
}
impl Spreadable for bool {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::Boolean))
    }
}