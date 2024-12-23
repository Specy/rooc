use core::fmt;

use crate::math::{BinOp, UnOp};
use crate::parser::model_transformer::TransformError;

use super::primitive::{Primitive, PrimitiveKind};

/// Trait for types that can have operators applied to them.
///
/// Defines operations for applying binary and unary operators to values,
/// as well as checking operator compatibility.
pub trait ApplyOp {
    type Target;
    type TargetType;
    type Error;
    fn apply_binary_op(&self, op: BinOp, to: &Self::Target) -> Result<Self::Target, Self::Error>;
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error>;
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool;
    fn can_apply_unary_op(op: UnOp) -> bool;
}

/// Trait for types that can be spread into a sequence of primitives.
///
/// Implementors can convert themselves into a vector of primitive values.
pub trait Spreadable {
    /// Converts self into a vector of primitives.
    ///
    /// # Returns
    /// * `Ok(Vec<Primitive>)` - The sequence of primitives
    /// * `Err(TransformError)` - If the value cannot be spread
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError>;
}

/// Represents errors that can occur during operator application.
#[derive(Debug)]
pub enum OperatorError {
    /// The operand type was incompatible with the operator
    IncompatibleType {
        operator: BinOp,
        expected: PrimitiveKind,
        found: PrimitiveKind,
    },
    /// The binary operator is not supported for the type
    UnsupportedBinOperation {
        operator: BinOp,
        found: PrimitiveKind,
    },
    /// The unary operator is not supported for the type
    UnsupportedUnOperation {
        operator: UnOp,
        found: PrimitiveKind,
    },
    /// An undefined value was used in an operation
    UndefinedUse,
}

impl OperatorError {
    /// Creates a new incompatible type error.
    ///
    /// # Arguments
    /// * `op` - The binary operator that was used
    /// * `expected` - The expected primitive type
    /// * `found` - The actual primitive type
    pub fn incompatible_type(op: BinOp, expected: PrimitiveKind, found: PrimitiveKind) -> Self {
        OperatorError::IncompatibleType {
            operator: op,
            expected,
            found,
        }
    }

    /// Creates a new unsupported binary operation error.
    ///
    /// # Arguments
    /// * `op` - The unsupported binary operator
    /// * `found` - The primitive type that doesn't support the operator
    pub fn unsupported_bin_operation(op: BinOp, found: PrimitiveKind) -> Self {
        OperatorError::UnsupportedBinOperation {
            operator: op,
            found,
        }
    }

    /// Creates a new unsupported unary operation error.
    ///
    /// # Arguments
    /// * `op` - The unsupported unary operator
    /// * `found` - The primitive type that doesn't support the operator
    pub fn unsupported_un_operation(op: UnOp, found: PrimitiveKind) -> Self {
        OperatorError::UnsupportedUnOperation {
            operator: op,
            found,
        }
    }
}

impl fmt::Display for OperatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OperatorError::IncompatibleType {
                expected,
                found,
                operator,
            } => format!(
                "Incompatible types for operator \"{}\", expected \"{}\", found \"{}\"",
                operator, expected, found
            ),
            OperatorError::UnsupportedBinOperation { operator, found } => format!(
                "Unsupported binary operation \"{}\" for type \"{}\"",
                operator, found
            ),
            OperatorError::UnsupportedUnOperation { operator, found } => format!(
                "Unsupported unary operation \"{}\" for type \"{}\"",
                operator, found
            ),
            OperatorError::UndefinedUse => "Used \"Undefined\" in operation".to_string(),
        };
        f.write_str(&s)
    }
}

impl ApplyOp for Primitive {
    type Target = Primitive;
    type TargetType = PrimitiveKind;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Self::Target) -> Result<Primitive, OperatorError> {
        match self {
            Primitive::Boolean(b) => b.apply_binary_op(op, to),
            Primitive::String(s) => s.apply_binary_op(op, to),
            Primitive::Tuple(t) => t.apply_binary_op(op, to),
            Primitive::GraphNode(gn) => gn.apply_binary_op(op, to),
            Primitive::GraphEdge(ge) => ge.apply_binary_op(op, to),
            Primitive::Graph(g) => g.apply_binary_op(op, to),
            Primitive::Iterable(i) => i.apply_binary_op(op, to),
            Primitive::Number(i) => i.apply_binary_op(op, to),
            Primitive::Integer(i) => i.apply_binary_op(op, to),
            Primitive::PositiveInteger(i) => i.apply_binary_op(op, to),
            Primitive::Undefined => Err(OperatorError::UndefinedUse),
        }
    }
    fn can_apply_binary_op(_: BinOp, _: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(_: UnOp) -> bool {
        false
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        match self {
            Primitive::Boolean(b) => b.apply_unary_op(op),
            Primitive::String(s) => s.apply_unary_op(op),
            Primitive::Tuple(t) => t.apply_unary_op(op),
            Primitive::GraphNode(gn) => gn.apply_unary_op(op),
            Primitive::GraphEdge(ge) => ge.apply_unary_op(op),
            Primitive::Graph(g) => g.apply_unary_op(op),
            Primitive::Iterable(i) => i.apply_unary_op(op),
            Primitive::Number(i) => i.apply_unary_op(op),
            Primitive::Integer(i) => i.apply_unary_op(op),
            Primitive::PositiveInteger(i) => i.apply_unary_op(op),
            Primitive::Undefined => Err(OperatorError::UndefinedUse),
        }
    }
}

impl Spreadable for Primitive {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        match self {
            Primitive::Boolean(b) => b.to_primitive_set(),
            Primitive::String(s) => s.to_primitive_set(),
            Primitive::Tuple(t) => t.to_primitive_set(),
            Primitive::GraphNode(gn) => gn.to_primitive_set(),
            Primitive::GraphEdge(ge) => ge.to_primitive_set(),
            Primitive::Graph(g) => g.to_primitive_set(),
            Primitive::Iterable(i) => i.to_primitive_set(),
            Primitive::Number(i) => i.to_primitive_set(),
            Primitive::Integer(i) => i.to_primitive_set(),
            Primitive::PositiveInteger(i) => i.to_primitive_set(),
            Primitive::Undefined => Err(TransformError::Unspreadable(PrimitiveKind::Undefined)),
        }
    }
}
