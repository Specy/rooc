use core::fmt;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::{Deserialize, Serialize};

use crate::math::{BinOp, UnOp};
use crate::parser::model_transformer::TransformError;
use crate::traits::ToLatex;

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

/// A tuple type that holds an ordered collection of `Primitive` values.
/// 
/// Tuples are immutable sequences that can store different types of primitives.
/// They are primarily used for grouping related values together.
///
/// # Example
/// ```
/// use rooc::{Primitive, Tuple};
/// 
/// let primitives = vec![
///     Primitive::Integer(1),
///     Primitive::String("hello".to_string())
/// ];
/// let tuple = Tuple::new(primitives);
/// ```
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

    /// Retrieves a reference to the `Primitive` at the specified index.
    ///
    /// # Arguments
    /// * `index` - The index of the element to retrieve
    ///
    /// # Returns
    /// `Some(&Primitive)` if the index exists, `None` otherwise
    pub fn get(&self, index: usize) -> Option<&Primitive> {
        self.0.get(index)
    }

    /// Retrieves a mutable reference to the `Primitive` at the specified index.
    ///
    /// # Arguments
    /// * `index` - The index of the element to retrieve
    ///
    /// # Returns
    /// `Some(&mut Primitive)` if the index exists, `None` otherwise
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Primitive> {
        self.0.get_mut(index)
    }

    /// Consumes the tuple and returns the underlying vector of primitives.
    ///
    /// # Returns
    /// The vector of `Primitive` values stored in the tuple
    pub fn into_primitives(self) -> Vec<Primitive> {
        self.0
    }

    /// Returns a reference to the underlying vector of primitives.
    ///
    /// # Returns
    /// A reference to the vector of `Primitive` values
    pub fn primitives(&self) -> &Vec<Primitive> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the type of this tuple as a `PrimitiveKind`.
    ///
    /// # Returns
    /// A `PrimitiveKind::Tuple` containing the types of all elements
    pub fn get_type(&self) -> PrimitiveKind {
        PrimitiveKind::Tuple(self.inner_types())
    }

    /// Returns a vector containing the `PrimitiveKind` of each element in the tuple.
    ///
    /// # Returns
    /// A vector of `PrimitiveKind` values representing the type of each element
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
