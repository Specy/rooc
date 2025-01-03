use core::fmt;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    graph::{Graph, GraphEdge, GraphNode},
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
    tuple::Tuple,
};
use crate::iterable_utils::flatten_primitive_array_values;
use crate::parser::model_transformer::TransformError;
use crate::traits::ToLatex;
use crate::{
    check_bounds,
    math::{BinOp, UnOp},
};
/// Represents different types of iterable collections in the system.
///
/// Each variant stores a vector of values of a specific primitive type.
/// This allows for type-safe iteration and operations over collections
/// of homogeneous elements.
///
/// # Example
/// ```rust
/// use rooc::IterableKind;
///
/// let numbers = IterableKind::Numbers(vec![1.0, 2.0, 3.0]);
/// let strings = IterableKind::Strings(vec!["a".to_string(), "b".to_string()]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum IterableKind {
    /// Collection of floating point numbers
    Numbers(Vec<f64>),
    /// Collection of signed integers
    Integers(Vec<i64>),
    /// Collection of unsigned integers
    PositiveIntegers(Vec<u64>),
    /// Collection of strings
    Strings(Vec<String>),
    /// Collection of graph edges
    Edges(Vec<GraphEdge>),
    /// Collection of graph nodes
    Nodes(Vec<GraphNode>),
    /// Collection of graphs
    Graphs(Vec<Graph>),
    /// Collection of tuples
    Tuples(Vec<Tuple>),
    /// Collection of boolean values
    Booleans(Vec<bool>),
    /// Nested collection of iterables
    Iterables(Vec<IterableKind>),
    /// Collection of any primitive type
    Anys(Vec<Primitive>),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IIterableKind: &'static str = r#"
export type SerializedIterable = 
    | { type: 'Numbers', value: number[] }
    | { type: 'Integers', value: number[] }
    | { type: 'PositiveIntegers', value: number[] }
    | { type: 'Strings', value: string[] }
    | { type: 'Edges', value: SerializedGraphEdge[] }
    | { type: 'Nodes', value: SerializedGraphNode[] }
    | { type: 'Graphs', value: SerializedGraph[] }
    | { type: 'Tuples', value: SerializedTuple[] }
    | { type: 'Booleans', value: boolean[] }
    | { type: 'Iterables', value: SerializedIterable[] }
    | { type: 'Anys', value: SerializedPrimitive[] }
"#;

impl IterableKind {
    /// Gets the primitive type of this iterable collection.
    ///
    /// # Returns
    /// A `PrimitiveKind::Iterable` containing the type of elements in the collection
    pub fn get_type(&self) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(self.inner_type()))
    }

    /// Converts this iterable into a primitive value.
    pub fn into_primitive(self) -> Primitive {
        Primitive::Iterable(self)
    }

    /// tries to flatten an array of primitives into an easier form
    pub fn flatten(self) -> IterableKind {
        match self {
            IterableKind::Anys(v) => flatten_primitive_array_values(v),
            _ => self,
        }
    }

    /// Gets the type of elements contained in this iterable.
    ///
    /// For nested iterables, returns the type of the innermost elements.
    pub fn inner_type(&self) -> PrimitiveKind {
        match self {
            IterableKind::Numbers(_) => PrimitiveKind::Number,
            IterableKind::Integers(_) => PrimitiveKind::Integer,
            IterableKind::PositiveIntegers(_) => PrimitiveKind::PositiveInteger,
            IterableKind::Strings(_) => PrimitiveKind::String,
            IterableKind::Edges(_) => PrimitiveKind::GraphEdge,
            IterableKind::Nodes(_) => PrimitiveKind::GraphNode,
            IterableKind::Anys(_) => PrimitiveKind::Any,
            IterableKind::Tuples(t) => t
                .first()
                .map(|e| e.get_type())
                .unwrap_or(PrimitiveKind::Undefined),
            IterableKind::Booleans(_) => PrimitiveKind::Boolean,
            IterableKind::Graphs(_) => PrimitiveKind::Graph,
            IterableKind::Iterables(i) => PrimitiveKind::Iterable(
                i.first()
                    .map(|e| e.inner_type())
                    .unwrap_or(PrimitiveKind::Undefined)
                    .into(),
            ),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            IterableKind::Numbers(v) => v.len(),
            IterableKind::Integers(v) => v.len(),
            IterableKind::PositiveIntegers(v) => v.len(),
            IterableKind::Strings(v) => v.len(),
            IterableKind::Edges(v) => v.len(),
            IterableKind::Nodes(v) => v.len(),
            IterableKind::Tuples(v) => v.len(),
            IterableKind::Iterables(v) => v.len(),
            IterableKind::Booleans(v) => v.len(),
            IterableKind::Graphs(v) => v.len(),
            IterableKind::Anys(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts this iterable into a vector of primitive values.
    pub fn to_primitives(self) -> Vec<Primitive> {
        match self {
            IterableKind::Numbers(v) => v.iter().map(|n| Primitive::Number(*n)).collect(),
            IterableKind::Integers(v) => v.iter().map(|n| Primitive::Integer(*n)).collect(),
            IterableKind::PositiveIntegers(v) => {
                v.iter().map(|n| Primitive::PositiveInteger(*n)).collect()
            }
            IterableKind::Anys(v) => v,
            IterableKind::Strings(v) => v
                .into_iter()
                .map(|s| Primitive::String((*s).to_string()))
                .collect(),
            IterableKind::Edges(v) => v
                .iter()
                .map(|e| Primitive::GraphEdge(e.to_owned()))
                .collect(),
            IterableKind::Nodes(v) => v.into_iter().map(Primitive::GraphNode).collect(),
            IterableKind::Tuples(v) => v.into_iter().map(Primitive::Tuple).collect(),
            IterableKind::Iterables(v) => v.into_iter().map(Primitive::Iterable).collect(),
            IterableKind::Booleans(v) => v.into_iter().map(Primitive::Boolean).collect(),
            IterableKind::Graphs(v) => v.into_iter().map(Primitive::Graph).collect(),
        }
    }

    /// Reads a value from the iterable at the specified indexes.
    ///
    /// For nested iterables, the indexes specify the path to the desired element.
    ///
    /// # Arguments
    /// * `indexes` - Vector of indexes specifying the path to the desired element
    ///
    /// # Returns
    /// * `Ok(Primitive)` - The value at the specified indexes
    /// * `Err(TransformError)` - If the indexes are out of bounds
    pub fn read(&self, indexes: Vec<usize>) -> Result<Primitive, TransformError> {
        if indexes.is_empty() {
            return Ok(Primitive::Undefined);
        }

        let mut current = self;
        let mut indexes = indexes;
        while !indexes.is_empty() {
            let i = indexes.remove(0);
            let ended = indexes.is_empty();
            if ended {
                let val = match current {
                    IterableKind::Booleans(v) => {
                        check_bounds!(i, v, self, Primitive::Boolean(v[i]))
                    }
                    IterableKind::Anys(v) => check_bounds!(i, v, self, v[i].clone()),
                    IterableKind::Numbers(v) => check_bounds!(i, v, self, Primitive::Number(v[i])),
                    IterableKind::Integers(v) => {
                        check_bounds!(i, v, self, Primitive::Integer(v[i]))
                    }
                    IterableKind::PositiveIntegers(v) => {
                        check_bounds!(i, v, self, Primitive::PositiveInteger(v[i]))
                    }
                    IterableKind::Strings(v) => {
                        check_bounds!(i, v, self, Primitive::String(v[i].to_string()))
                    }
                    IterableKind::Edges(v) => {
                        check_bounds!(i, v, self, Primitive::GraphEdge(v[i].to_owned()))
                    }
                    IterableKind::Nodes(v) => {
                        check_bounds!(i, v, self, Primitive::GraphNode(v[i].to_owned()))
                    }
                    IterableKind::Tuples(v) => {
                        check_bounds!(i, v, self, Primitive::Tuple(v[i].clone()))
                    }
                    IterableKind::Iterables(v) => {
                        check_bounds!(i, v, self, Primitive::Iterable(v[i].clone()))
                    }
                    IterableKind::Graphs(v) => {
                        check_bounds!(i, v, self, Primitive::Graph(v[i].clone()))
                    }
                };
                return Ok(val);
            } else {
                match current {
                    IterableKind::Iterables(v) => {
                        if i < v.len() {
                            current = &v[i];
                        } else {
                            return Err(TransformError::OutOfBounds(format!(
                                "cannot access index {} of {}",
                                i, self
                            )));
                        }
                    }
                    _ => {
                        return Err(TransformError::OutOfBounds(format!(
                            "cannot access index {} of {}",
                            i, self
                        )));
                    }
                }
            }
        }
        Err(TransformError::OutOfBounds(format!(
            "cannot access index {} of {}",
            indexes[0], self
        )))
    }

    /// Returns the nesting depth of this iterable.
    ///
    /// For non-nested iterables, returns 1.
    /// For nested iterables, returns the maximum nesting depth.
    pub fn depth(&self) -> usize {
        let mut current = self;
        let mut depth = 1;
        while let IterableKind::Iterables(v) = current {
            depth += 1;
            match v.first() {
                Some(i) => current = i,
                None => break,
            }
        }
        depth
    }

    /// Returns a string representation of the iterable with proper indentation.
    ///
    /// # Arguments
    /// * `depth` - The current indentation depth
    pub fn to_string_depth(&self, depth: usize) -> String {
        match self {
            IterableKind::Iterables(v) => {
                let s = v
                    .iter()
                    .map(|e| e.to_string_depth(depth + 1))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}[\n{}\n]", "    ".repeat(depth), s)
            }
            _ => format!("{}{}", "    ".repeat(depth), self),
        }
    }

    /// Returns a LaTeX representation of the iterable.
    ///
    /// # Arguments
    /// * `include_block` - Whether to wrap the output in a matrix block
    pub fn latexify(&self, include_block: bool) -> String {
        match self {
            IterableKind::Numbers(v) => latexify_vec(v, include_block),
            IterableKind::Integers(v) => latexify_vec(v, include_block),
            IterableKind::PositiveIntegers(v) => latexify_vec(v, include_block),
            IterableKind::Anys(v) => latexify_vec(v, include_block),
            IterableKind::Strings(v) => latexify_vec(v, include_block),
            IterableKind::Edges(v) => latexify_vec(v, include_block),
            IterableKind::Nodes(v) => latexify_vec(v, include_block),
            IterableKind::Tuples(v) => latexify_vec(v, include_block),
            IterableKind::Booleans(v) => latexify_vec(v, include_block),
            IterableKind::Graphs(v) => latexify_vec(v, include_block),
            IterableKind::Iterables(v) => {
                let s = v
                    .iter()
                    .map(|i| i.to_latex())
                    .collect::<Vec<_>>()
                    .join("\\\\");
                if include_block {
                    format!("\\begin{{bmatrix}} {} \\end{{bmatrix}}", s)
                } else {
                    s.to_string()
                }
            }
        }
    }
}
fn latexify_vec<T>(v: &[T], include_block: bool) -> String
where
    T: ToLatex,
{
    let values = v
        .iter()
        .map(|e| e.to_latex())
        .collect::<Vec<_>>()
        .join(" & ");
    if include_block {
        format!("\\begin{{bmatrix}} {} \\end{{bmatrix}}", values)
    } else {
        values.to_string()
    }
}

impl ToLatex for IterableKind {
    fn to_latex(&self) -> String {
        match self {
            IterableKind::Iterables(v) => {
                let depth = self.depth();
                if depth == 2 {
                    //try to prettify for 2d matrices
                    let items = v
                        .iter()
                        .map(|i| i.latexify(false))
                        .collect::<Vec<_>>()
                        .join(" \\\\ ");
                    format!("\\begin{{bmatrix}} {} \\end{{bmatrix}}", items)
                } else {
                    self.latexify(true)
                }
            }
            _ => self.latexify(true),
        }
    }
}

impl fmt::Display for IterableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //TODO should i turn this into a self.to_primitive_set()  and then iterate and stringify?
        let s = match self {
            IterableKind::Numbers(v) => format!("{:?}", v),
            IterableKind::Integers(v) => format!("{:?}", v),
            IterableKind::Anys(v) => format!("{:?}", v),
            IterableKind::PositiveIntegers(v) => format!("{:?}", v),
            IterableKind::Strings(v) => format!("{:?}", v),
            IterableKind::Edges(v) => format!("{:?}", v),
            IterableKind::Nodes(v) => format!("{:?}", v),
            IterableKind::Tuples(v) => format!("{:?}", v),
            IterableKind::Booleans(v) => format!("{:?}", v),
            IterableKind::Graphs(v) => format!("{:?}", v),
            IterableKind::Iterables(v) => {
                let result = v
                    .iter()
                    .map(|i| i.to_string_depth(1))
                    .collect::<Vec<_>>()
                    .join(",\n");
                format!("[\n{}\n]", result)
            }
        };
        f.write_str(&s)
    }
}

impl ApplyOp for IterableKind {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(op, _to.get_type()))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            self.inner_type(),
        ))
    }
    fn can_apply_binary_op(_: BinOp, _: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(_: UnOp) -> bool {
        false
    }
}

impl Spreadable for IterableKind {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Ok(self.to_primitives())
    }
}
