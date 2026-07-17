use crate::math::{BinOp, UnOp};
use crate::parser::model_transformer::TransformError;

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

/// Wraps a checked i64 result, turning overflow (`None`) into an `Overflow` error
/// instead of panicking (debug) or silently wrapping (release).
fn checked_i64(res: Option<i64>, op: BinOp) -> Result<Primitive, OperatorError> {
    res.map(Primitive::Integer)
        .ok_or_else(|| OperatorError::overflow(op, PrimitiveKind::Integer))
}

/// Wraps a checked u64 result, turning overflow (`None`) into an `Overflow` error.
fn checked_u64(res: Option<u64>, op: BinOp) -> Result<Primitive, OperatorError> {
    res.map(Primitive::PositiveInteger)
        .ok_or_else(|| OperatorError::overflow(op, PrimitiveKind::PositiveInteger))
}

/// Performs a floating-point division, returning a `DivisionByZero` error instead of
/// silently producing `inf`/`NaN` (which would poison the compiled model).
fn checked_div(a: f64, b: f64) -> Result<Primitive, OperatorError> {
    if b == 0.0 {
        Err(OperatorError::division_by_zero())
    } else {
        Ok(Primitive::Number(a / b))
    }
}

/* --------- ApplyOp --------- */

impl ApplyOp for String {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match to {
            Primitive::String(s) => match op {
                BinOp::Add => Ok(Primitive::String(format!("{}{}", self, s))),
                op @ (BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::And
                | BinOp::Or
                | BinOp::Xor
                | BinOp::Implies
                | BinOp::Iff) => Err(OperatorError::unsupported_bin_operation(
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
        match op {
            BinOp::Add => matches!(to, PrimitiveKind::String),
            BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::And
            | BinOp::Or
            | BinOp::Xor
            | BinOp::Implies
            | BinOp::Iff => false,
        }
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        match op {
            UnOp::Neg | UnOp::Not => false,
        }
    }
}

impl ApplyOp for bool {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match (op, to) {
            (BinOp::And, Primitive::Boolean(b)) => Ok(Primitive::Boolean(*self && *b)),
            (BinOp::Or, Primitive::Boolean(b)) => Ok(Primitive::Boolean(*self || *b)),
            (BinOp::Xor, Primitive::Boolean(b)) => Ok(Primitive::Boolean(*self != *b)),
            (BinOp::Implies, Primitive::Boolean(b)) => Ok(Primitive::Boolean(!*self || *b)),
            (BinOp::Iff, Primitive::Boolean(b)) => Ok(Primitive::Boolean(*self == *b)),
            //booleans take part in arithmetic as 0/1 values
            (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div, _) => {
                (*self as u8 as f64).apply_binary_op(op, to)
            }
            (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff, to) => Err(
                OperatorError::incompatible_type(op, PrimitiveKind::Boolean, to.get_type()),
            ),
        }
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        match op {
            UnOp::Not => Ok(Primitive::Boolean(!*self)),
            //negation follows the 0/1 arithmetic view of booleans
            UnOp::Neg => Ok(Primitive::Number(-(*self as u8 as f64))),
        }
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        match op {
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => {
                matches!(to, PrimitiveKind::Boolean)
            }
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => matches!(
                to,
                PrimitiveKind::Number
                    | PrimitiveKind::Integer
                    | PrimitiveKind::PositiveInteger
                    | PrimitiveKind::Boolean
            ),
        }
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        match op {
            UnOp::Not => true,
            UnOp::Neg => true,
        }
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
                BinOp::Div => checked_div(*self, *n),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Number),
                ),
            },
            Primitive::Integer(n) => match op {
                BinOp::Add => Ok(Primitive::Number(*self + (*n as f64))),
                BinOp::Sub => Ok(Primitive::Number(*self - (*n as f64))),
                BinOp::Mul => Ok(Primitive::Number(*self * (*n as f64))),
                BinOp::Div => checked_div(*self, *n as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Number),
                ),
            },
            Primitive::PositiveInteger(n) => match op {
                BinOp::Add => Ok(Primitive::Number(*self + (*n as f64))),
                BinOp::Sub => Ok(Primitive::Number(*self - (*n as f64))),
                BinOp::Mul => Ok(Primitive::Number(*self * (*n as f64))),
                BinOp::Div => checked_div(*self, *n as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Number),
                ),
            },
            Primitive::Boolean(n) => match op {
                BinOp::Add => Ok(Primitive::Number(*self + (*n as i8 as f64))),
                BinOp::Sub => Ok(Primitive::Number(*self - (*n as i8 as f64))),
                BinOp::Mul => Ok(Primitive::Number(*self * (*n as i8 as f64))),
                BinOp::Div => checked_div(*self, *n as i8 as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Number),
                ),
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
            UnOp::Not => Err(OperatorError::unsupported_un_operation(
                op,
                PrimitiveKind::Number,
            )),
        }
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => matches!(
                to,
                PrimitiveKind::Number
                    | PrimitiveKind::Integer
                    | PrimitiveKind::PositiveInteger
                    | PrimitiveKind::Boolean
            ),
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => false,
        }
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        match op {
            UnOp::Neg => true,
            UnOp::Not => false,
        }
    }
}

impl ApplyOp for i64 {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match to {
            Primitive::Integer(n) => match op {
                BinOp::Add => checked_i64(self.checked_add(*n), BinOp::Add),
                BinOp::Sub => checked_i64(self.checked_sub(*n), BinOp::Sub),
                BinOp::Mul => checked_i64(self.checked_mul(*n), BinOp::Mul),
                BinOp::Div => checked_div(*self as f64, *n as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Integer),
                ),
            },
            Primitive::Number(n) => match op {
                BinOp::Add => Ok(Primitive::Number((*self as f64) + n)),
                BinOp::Sub => Ok(Primitive::Number((*self as f64) - n)),
                BinOp::Mul => Ok(Primitive::Number((*self as f64) * n)),
                BinOp::Div => checked_div(*self as f64, *n),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Integer),
                ),
            },
            Primitive::PositiveInteger(n) => match op {
                BinOp::Add => checked_i64(self.checked_add(*n as i64), BinOp::Add),
                BinOp::Sub => checked_i64(self.checked_sub(*n as i64), BinOp::Sub),
                BinOp::Mul => checked_i64(self.checked_mul(*n as i64), BinOp::Mul),
                BinOp::Div => checked_div(*self as f64, *n as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Integer),
                ),
            },
            Primitive::Boolean(n) => match op {
                BinOp::Add => checked_i64(self.checked_add(*n as i64), BinOp::Add),
                BinOp::Sub => checked_i64(self.checked_sub(*n as i64), BinOp::Sub),
                BinOp::Mul => checked_i64(self.checked_mul(*n as i64), BinOp::Mul),
                BinOp::Div => checked_div(*self as f64, *n as u8 as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::Integer),
                ),
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
            UnOp::Not => Err(OperatorError::unsupported_un_operation(
                op,
                PrimitiveKind::Integer,
            )),
        }
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => matches!(
                to,
                PrimitiveKind::Number
                    | PrimitiveKind::Integer
                    | PrimitiveKind::PositiveInteger
                    | PrimitiveKind::Boolean
            ),
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => false,
        }
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        match op {
            UnOp::Neg => true,
            UnOp::Not => false,
        }
    }
}

impl ApplyOp for u64 {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
        match to {
            Primitive::PositiveInteger(n) => match op {
                BinOp::Add => checked_u64(self.checked_add(*n), BinOp::Add),
                BinOp::Sub => checked_i64((*self as i64).checked_sub(*n as i64), BinOp::Sub),
                BinOp::Mul => checked_u64(self.checked_mul(*n), BinOp::Mul),
                BinOp::Div => checked_div(*self as f64, *n as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::PositiveInteger),
                ),
            },
            Primitive::Integer(n) => match op {
                BinOp::Add => checked_i64((*self as i64).checked_add(*n), BinOp::Add),
                BinOp::Sub => checked_i64((*self as i64).checked_sub(*n), BinOp::Sub),
                BinOp::Mul => checked_i64((*self as i64).checked_mul(*n), BinOp::Mul),
                BinOp::Div => checked_div(*self as f64, *n as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::PositiveInteger),
                ),
            },
            Primitive::Number(n) => match op {
                BinOp::Add => Ok(Primitive::Number((*self as f64) + n)),
                BinOp::Sub => Ok(Primitive::Number((*self as f64) - n)),
                BinOp::Mul => Ok(Primitive::Number((*self as f64) * n)),
                BinOp::Div => checked_div(*self as f64, *n),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::PositiveInteger),
                ),
            },
            Primitive::Boolean(n) => match op {
                BinOp::Add => checked_u64(self.checked_add(*n as u64), BinOp::Add),
                BinOp::Sub => checked_i64((*self as i64).checked_sub(*n as i64), BinOp::Sub),
                BinOp::Mul => checked_u64(self.checked_mul(*n as u64), BinOp::Mul),
                BinOp::Div => checked_div(*self as f64, *n as u8 as f64),
                op @ (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff) => Err(
                    OperatorError::unsupported_bin_operation(op, PrimitiveKind::PositiveInteger),
                ),
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
            UnOp::Not => Err(OperatorError::unsupported_un_operation(
                op,
                PrimitiveKind::PositiveInteger,
            )),
        }
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => matches!(
                to,
                PrimitiveKind::Number
                    | PrimitiveKind::Integer
                    | PrimitiveKind::PositiveInteger
                    | PrimitiveKind::Boolean
            ),
            BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Implies | BinOp::Iff => false,
        }
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        match op {
            UnOp::Neg => true,
            UnOp::Not => false,
        }
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
