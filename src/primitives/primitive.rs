use core::fmt;
use std::fmt::Display;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    graph::{Graph, GraphEdge, GraphNode},
    iterable::IterableKind,
    tuple::Tuple,
};
use crate::math::{float_lt, float_ne};
use crate::parser::model_transformer::TransformError;
use crate::traits::ToLatex;
use crate::{
    bail_wrong_argument, match_or_bail,
    math::{BinOp, UnOp},
    primitives::primitive_traits::ApplyOp,
    wrong_argument,
};

/// Represents a primitive value in the system.
///
/// This enum contains all possible primitive types that can be used in expressions and calculations.
/// Each variant represents a different type of value with its associated data.
///
/// # Example
/// ```rust
/// use rooc::Primitive;
///
/// let num = Primitive::Number(42.0);
/// let text = Primitive::String("Hello".to_string());
/// let flag = Primitive::Boolean(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum Primitive {
    /// A floating point number
    Number(f64),
    /// A signed integer
    Integer(i64),
    /// An unsigned integer
    PositiveInteger(u64),
    /// A text string
    String(String),
    /// An iterable collection of values
    Iterable(IterableKind),
    /// A graph structure
    Graph(Graph),
    /// An edge in a graph
    GraphEdge(GraphEdge),
    /// A node in a graph
    GraphNode(GraphNode),
    /// An ordered collection of primitives
    Tuple(Tuple),
    /// A boolean value
    Boolean(bool),
    /// Represents an undefined value
    Undefined,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IPrimitive: &'static str = r#"
export type SerializedPrimitive =
    | { type: 'Number', value: number }
    | { type: 'Integer', value: number }
    | { type: 'PositiveInteger', value: number }
    | { type: 'String', value: string }
    | { type: 'Iterable', value: SerializedIterable }
    | { type: 'Graph', value: SerializedGraph }
    | { type: 'GraphEdge', value: SerializedGraphEdge }
    | { type: 'GraphNode', value: SerializedGraphNode }
    | { type: 'Tuple', value: SerializedTuple }
    | { type: 'Boolean', value: boolean }
    | { type: 'Undefined' }
"#;

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PrimitiveKind {
    /// Floating point number type
    Number,
    /// Signed integer type
    Integer,
    /// Unsigned integer type
    PositiveInteger,
    /// String type
    String,
    /// Iterable type containing elements of the specified kind
    Iterable(Box<PrimitiveKind>),
    /// Graph type
    Graph,
    /// Graph edge type
    GraphEdge,
    /// Graph node type
    GraphNode,
    /// Tuple type containing a sequence of primitive kinds
    Tuple(Vec<PrimitiveKind>),
    /// Boolean type
    Boolean,
    /// Undefined type
    Undefined,
    /// Any type (used for type checking)
    Any,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IPrimitiveKind: &'static str = r#"
export type SerializedPrimitiveKind =
    | { type: 'Number' }
    | { type: 'Integer' }
    | { type: 'PositiveInteger' }
    | { type: 'String' }
    | { type: 'Iterable', value: SerializedPrimitiveKind }
    | { type: 'Graph' }
    | { type: 'GraphEdge' }
    | { type: 'GraphNode' }
    | { type: 'Tuple', value: SerializedPrimitiveKind[] }
    | { type: 'Boolean' }
    | { type: 'Undefined' }
    | { type: 'Any' }
"#;

impl PrimitiveKind {
    /// Creates a PrimitiveKind from a Primitive value.
    ///
    /// # Arguments
    /// * `p` - The primitive value to get the type from
    ///
    /// # Returns
    /// The corresponding PrimitiveKind for the given Primitive
    pub fn from_primitive(p: &Primitive) -> Self {
        match p {
            Primitive::Number(_) => PrimitiveKind::Number,
            Primitive::Integer(_) => PrimitiveKind::Integer,
            Primitive::PositiveInteger(_) => PrimitiveKind::PositiveInteger,
            Primitive::String(_) => PrimitiveKind::String,
            Primitive::Iterable(p) => p.get_type(),
            Primitive::Graph(_) => PrimitiveKind::Graph,
            Primitive::GraphEdge(_) => PrimitiveKind::GraphEdge,
            Primitive::GraphNode(_) => PrimitiveKind::GraphNode,
            Primitive::Tuple(t) => t.get_type(),
            Primitive::Boolean(_) => PrimitiveKind::Boolean,
            Primitive::Undefined => PrimitiveKind::Undefined,
        }
    }

    /// Checks if the type is numeric (Number, Integer, PositiveInteger, or Boolean).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            PrimitiveKind::Number
                | PrimitiveKind::Integer
                | PrimitiveKind::PositiveInteger
                | PrimitiveKind::Boolean
        )
    }
    pub fn is_any(&self) -> bool {
        matches!(self, PrimitiveKind::Any)
    }
    pub fn is_iterable(&self) -> bool {
        matches!(self, PrimitiveKind::Iterable(_))
    }

    /// Returns the types that this primitive kind can be spread into.
    ///
    /// # Returns
    /// * `Ok(Vec<PrimitiveKind>)` - The types this kind can be spread into
    /// * `Err(TransformError)` - If the type cannot be spread
    pub fn can_spread_into(&self) -> Result<Vec<PrimitiveKind>, TransformError> {
        match self {
            PrimitiveKind::Tuple(t) => Ok(t.clone()),
            PrimitiveKind::GraphEdge => Ok(vec![
                PrimitiveKind::String,
                PrimitiveKind::String,
                PrimitiveKind::Number,
            ]),
            //TODO PrimitiveKind::Any => Ok(vec![PrimitiveKind::Any; 4]),
            _ => Err(TransformError::Unspreadable(self.clone())),
        }
    }

    pub fn can_apply_binary_op(&self, op: BinOp, to: PrimitiveKind) -> bool {
        match self {
            PrimitiveKind::Any => true, //make it fail at runtime
            PrimitiveKind::Undefined => false,
            PrimitiveKind::Integer => i64::can_apply_binary_op(op, to),
            PrimitiveKind::PositiveInteger => u64::can_apply_binary_op(op, to),
            PrimitiveKind::Number => f64::can_apply_binary_op(op, to),
            PrimitiveKind::Boolean => bool::can_apply_binary_op(op, to),
            PrimitiveKind::Graph => Graph::can_apply_binary_op(op, to),
            PrimitiveKind::GraphEdge => GraphEdge::can_apply_binary_op(op, to),
            PrimitiveKind::GraphNode => GraphNode::can_apply_binary_op(op, to),
            PrimitiveKind::Tuple(_) => Tuple::can_apply_binary_op(op, to),
            PrimitiveKind::Iterable(_) => IterableKind::can_apply_binary_op(op, to),
            PrimitiveKind::String => String::can_apply_binary_op(op, to),
        }
    }
    pub fn can_apply_unary_op(&self, op: UnOp) -> bool {
        match self {
            PrimitiveKind::Any => true, //make it fail at runtime
            PrimitiveKind::Undefined => false,
            PrimitiveKind::Integer => i64::can_apply_unary_op(op),
            PrimitiveKind::PositiveInteger => u64::can_apply_unary_op(op),
            PrimitiveKind::Number => f64::can_apply_unary_op(op),
            PrimitiveKind::Boolean => bool::can_apply_unary_op(op),
            PrimitiveKind::Graph => Graph::can_apply_unary_op(op),
            PrimitiveKind::GraphEdge => GraphEdge::can_apply_unary_op(op),
            PrimitiveKind::GraphNode => GraphNode::can_apply_unary_op(op),
            PrimitiveKind::Tuple(_) => Tuple::can_apply_unary_op(op),
            PrimitiveKind::Iterable(_) => IterableKind::can_apply_unary_op(op),
            PrimitiveKind::String => String::can_apply_unary_op(op),
        }
    }
}

impl Display for PrimitiveKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PrimitiveKind::Number => "Number".to_string(),
            PrimitiveKind::String => "String".to_string(),
            PrimitiveKind::Integer => "Integer".to_string(),
            PrimitiveKind::PositiveInteger => "PositiveInteger".to_string(),
            PrimitiveKind::Iterable(i) => format!("{}[]", i),
            PrimitiveKind::Graph => "Graph".to_string(),
            PrimitiveKind::GraphEdge => "GraphEdge".to_string(),
            PrimitiveKind::GraphNode => "GraphNode".to_string(),
            PrimitiveKind::Tuple(t) => format!(
                "({})",
                t.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PrimitiveKind::Boolean => "Boolean".to_string(),
            PrimitiveKind::Undefined => "Undefined".to_string(),
            PrimitiveKind::Any => "Any".to_string(),
        };
        f.write_str(&s)
    }
}

impl Primitive {
    pub fn get_type(&self) -> PrimitiveKind {
        PrimitiveKind::from_primitive(self)
    }

    /// Gets a string representation of this primitive's type.
    pub fn type_string(&self) -> String {
        self.get_type().to_string()
    }
    pub fn as_number(&self) -> Result<f64, TransformError> {
        match_or_bail!(PrimitiveKind::Number,
            Primitive::Number(n) => Ok(*n)
            ; (self))
    }

    /// Attempts to get the value as a number, converting from compatible types.
    ///
    /// # Returns
    /// * `Ok(f64)` - The numeric value
    /// * `Err(TransformError)` - If the value cannot be converted to a number
    pub fn as_number_cast(&self) -> Result<f64, TransformError> {
        match self {
            Primitive::Number(n) => Ok(*n),
            Primitive::Integer(n) => Ok(*n as f64),
            Primitive::PositiveInteger(n) => Ok(*n as f64),
            Primitive::Boolean(b) => Ok(*b as u8 as f64),
            _ => bail_wrong_argument!(PrimitiveKind::Number, self),
        }
    }
    pub fn as_integer(&self) -> Result<i64, TransformError> {
        match_or_bail!(PrimitiveKind::Integer,
            Primitive::Integer(n) => Ok(*n)
            ; (self))
    }

    /// Attempts to get the value as an integer, converting from compatible types.
    ///
    /// # Returns
    /// * `Ok(i64)` - The integer value
    /// * `Err(TransformError)` - If the value cannot be converted to an integer
    pub fn as_integer_cast(&self) -> Result<i64, TransformError> {
        match self {
            Primitive::Integer(n) => Ok(*n),
            Primitive::PositiveInteger(n) => Ok(*n as i64),
            Primitive::Boolean(b) => Ok(*b as u8 as i64),
            Primitive::Number(n) => {
                if float_ne(n.fract(), 0.0) {
                    Err(wrong_argument!(PrimitiveKind::Integer, self))
                } else {
                    Ok(*n as i64)
                }
            }
            _ => bail_wrong_argument!(PrimitiveKind::Integer, self),
        }
    }
    pub fn as_usize(&self) -> Result<usize, TransformError> {
        match_or_bail!(PrimitiveKind::PositiveInteger,
            Primitive::PositiveInteger(n) => Ok(*n as usize)
            ; (self))
    }

    /// Attempts to get the value as a usize, converting from compatible types.
    ///
    /// # Returns
    /// * `Ok(usize)` - The unsigned size value
    /// * `Err(TransformError)` - If the value cannot be converted to a usize
    pub fn as_usize_cast(&self) -> Result<usize, TransformError> {
        match self {
            Primitive::PositiveInteger(n) => Ok(*n as usize),
            Primitive::Integer(n) => {
                if *n < 0 {
                    Err(wrong_argument!(PrimitiveKind::PositiveInteger, self))
                } else {
                    Ok(*n as usize)
                }
            }
            Primitive::Boolean(b) => Ok(*b as u8 as usize),
            Primitive::Number(n) => {
                if float_ne(n.fract(), 0.0) || float_lt(*n, 0.0) {
                    Err(wrong_argument!(PrimitiveKind::PositiveInteger, self))
                } else {
                    Ok(*n as usize)
                }
            }
            _ => bail_wrong_argument!(PrimitiveKind::PositiveInteger, self),
        }
    }
    pub fn as_positive_integer(&self) -> Result<u64, TransformError> {
        match_or_bail!(PrimitiveKind::PositiveInteger,
            Primitive::PositiveInteger(n) => Ok(*n)
            ; (self))
    }
    pub fn as_graph(&self) -> Result<&Graph, TransformError> {
        match_or_bail!(PrimitiveKind::Graph,
            Primitive::Graph(g) => Ok(g)
          ; (self))
    }
    pub fn as_graph_edge(&self) -> Result<&GraphEdge, TransformError> {
        match_or_bail!(PrimitiveKind::GraphEdge,
            Primitive::GraphEdge(e) => Ok(e)
          ; (self))
    }
    pub fn as_graph_node(&self) -> Result<&GraphNode, TransformError> {
        match_or_bail!(PrimitiveKind::GraphNode,
            Primitive::GraphNode(n) => Ok(n)
          ; (self))
    }
    pub fn as_boolean(&self) -> Result<bool, TransformError> {
        match_or_bail!(PrimitiveKind::Boolean,
            Primitive::Boolean(b) => Ok(*b)
          ; (self))
    }
    pub fn as_string(&self) -> Result<&String, TransformError> {
        match_or_bail!(PrimitiveKind::String,
            Primitive::String(s) => Ok(s)
          ; (self))
    }
    pub fn as_iterator(&self) -> Result<&IterableKind, TransformError> {
        match_or_bail!(
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Any)),
            Primitive::Iterable(i) => Ok(i);
            (self)
        )
    }
    pub fn as_tuple(&self) -> Result<&Vec<Primitive>, TransformError> {
        match_or_bail!(
            PrimitiveKind::Tuple(vec![]),
            Primitive::Tuple(t) => Ok(t.primitives());
            (self)
        )
    }
}

impl ToLatex for Primitive {
    fn to_latex(&self) -> String {
        match self {
            Primitive::Number(n) => n.to_latex(),
            Primitive::Integer(n) => n.to_latex(),
            Primitive::PositiveInteger(n) => n.to_latex(),
            Primitive::String(s) => s.to_latex(),
            Primitive::Iterable(i) => i.to_latex(),
            Primitive::Graph(g) => g.to_latex(),
            Primitive::GraphEdge(e) => e.to_latex(),
            Primitive::GraphNode(n) => n.to_latex(),
            Primitive::Tuple(v) => v.to_latex(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Undefined => "undefined".to_string(),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Primitive::Number(n) => n.to_string(),
            Primitive::Integer(n) => n.to_string(),
            Primitive::PositiveInteger(n) => n.to_string(),
            Primitive::String(s) => format!("\"{}\"", s),
            Primitive::Iterable(i) => i.to_string(),
            Primitive::Graph(g) => g.to_string(),
            Primitive::GraphEdge(e) => e.to_string(),
            Primitive::GraphNode(n) => n.to_string(),
            Primitive::Tuple(v) => format!("{:?}", v),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Undefined => "undefined".to_string(),
        };
        f.write_str(&s)
    }
}
