use crate::{math::operators::{BinOp, UnOp}, parser::transformer::TransformError};

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

/* --------- ApplyOp --------- */

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
            Primitive::Integer(n) => match op {
                BinOp::Add => Ok(Primitive::Number(*self + (*n as f64))),
                BinOp::Sub => Ok(Primitive::Number(*self - (*n as f64))),
                BinOp::Mul => Ok(Primitive::Number(*self * (*n as f64))),
                BinOp::Div => Ok(Primitive::Number(*self / (*n as f64))),
            },
            Primitive::PositiveInteger(n) => match op {
                BinOp::Add => Ok(Primitive::Number(*self + (*n as f64))),
                BinOp::Sub => Ok(Primitive::Number(*self - (*n as f64))),
                BinOp::Mul => Ok(Primitive::Number(*self * (*n as f64))),
                BinOp::Div => Ok(Primitive::Number(*self / (*n as f64))),
            },
            Primitive::Boolean(n) => match op {
                BinOp::Add => Ok(Primitive::Number(*self + (*n as i8 as f64))),
                BinOp::Sub => Ok(Primitive::Number(*self - (*n as i8 as f64))),
                BinOp::Mul => Ok(Primitive::Number(*self * (*n as i8 as f64))),
                BinOp::Div => Ok(Primitive::Number(*self / (*n as i8 as f64))),
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
        matches!(to, PrimitiveKind::Number | PrimitiveKind::Integer | PrimitiveKind::PositiveInteger | PrimitiveKind::Boolean)
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        matches!(op, UnOp::Neg)
    }
}


impl ApplyOp for i64 {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match to {
            Primitive::Integer(n) => match op {
                BinOp::Add => Ok(Primitive::Integer(self + n)),
                BinOp::Sub => Ok(Primitive::Integer(self - n)),
                BinOp::Mul => Ok(Primitive::Integer(self * n)),
                BinOp::Div => Ok(Primitive::Number((*self as f64) / (*n as f64))),
            },
            Primitive::Number(n) => match op {
                BinOp::Add => Ok(Primitive::Number((*self as f64) + n)),
                BinOp::Sub => Ok(Primitive::Number((*self as f64) - n)),
                BinOp::Mul => Ok(Primitive::Number((*self as f64) * n)),
                BinOp::Div => Ok(Primitive::Number((*self as f64) / n)),
            },
            Primitive::PositiveInteger(n) => match op {
                BinOp::Add => Ok(Primitive::Integer(*self + (*n as i64))),
                BinOp::Sub => Ok(Primitive::Integer(*self - (*n as i64))),
                BinOp::Mul => Ok(Primitive::Integer(*self * (*n as i64))),
                BinOp::Div => Ok(Primitive::Number((*self as f64) / (*n as f64))),
            },
            Primitive::Boolean(n) => match op {
                BinOp::Add => Ok(Primitive::Integer(*self + (*n as i64))),
                BinOp::Sub => Ok(Primitive::Integer(*self - (*n as i64))),
                BinOp::Mul => Ok(Primitive::Integer(*self)),
                BinOp::Div => Ok(Primitive::Integer(*self)),
            },
            _ => Err(OperatorError::incompatible_type(
                op,
                PrimitiveKind::Integer,
                to.get_type(),
            )),
        }
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        match op {
            UnOp::Neg => Ok(Primitive::Integer(-self)),
        }
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        matches!(to, PrimitiveKind::Number | PrimitiveKind::Integer | PrimitiveKind::PositiveInteger | PrimitiveKind::Boolean)
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        matches!(op, UnOp::Neg)
    }
}

impl ApplyOp for u64 {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match to {
            Primitive::PositiveInteger(n) => match op {
                BinOp::Add => Ok(Primitive::PositiveInteger(self + n)),
                BinOp::Sub => Ok(Primitive::Integer((*self as i64) - (*n as i64))),
                BinOp::Mul => Ok(Primitive::PositiveInteger(self * n)),
                BinOp::Div => Ok(Primitive::Number((*self as f64) / (*n as f64))),
            },
            Primitive::Integer(n) => match op {
                BinOp::Add => Ok(Primitive::Integer((*self as i64) + n)),
                BinOp::Sub => Ok(Primitive::Integer((*self as i64) - n)),
                BinOp::Mul => Ok(Primitive::Integer((*self as i64) * n)),
                BinOp::Div => Ok(Primitive::Number((*self as f64) / (*n as f64))),
            },
            Primitive::Number(n) => match op {
                BinOp::Add => Ok(Primitive::Number((*self as f64) + n)),
                BinOp::Sub => Ok(Primitive::Number((*self as f64) - n)),
                BinOp::Mul => Ok(Primitive::Number((*self as f64) * n)),
                BinOp::Div => Ok(Primitive::Number((*self as f64) / n)),
            },
            Primitive::Boolean(n) => match op {
                BinOp::Add => Ok(Primitive::PositiveInteger(*self + (*n as u64))),
                BinOp::Sub => Ok(Primitive::Integer((*self as i64) - (*n as i64))),
                BinOp::Mul => Ok(Primitive::PositiveInteger(*self)),
                BinOp::Div => Ok(Primitive::PositiveInteger(*self)),
            },
            _ => Err(OperatorError::incompatible_type(
                op,
                PrimitiveKind::PositiveInteger,
                to.get_type(),
            )),
        }
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        match op {
            UnOp::Neg => Ok(Primitive::Integer(-(*self as i64))),
        }
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        matches!(to, PrimitiveKind::Number | PrimitiveKind::Integer | PrimitiveKind::PositiveInteger | PrimitiveKind::Boolean)
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        matches!(op, UnOp::Neg)
    }
}


/* --------- Spreadable --------- */

impl Spreadable for f64 {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::Number))
    }
}

impl Spreadable for i64 {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::Integer))
    }
}

impl Spreadable for u64 {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::PositiveInteger))
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