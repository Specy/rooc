use core::fmt;
use std::collections::HashMap;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    math::operators::{BinOp, UnOp},
    parser::transformer::TransformError,
};

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub weight: Option<f64>,
}
#[wasm_bindgen(typescript_custom_section)]
const IGraphEdge: &'static str = r#"
export type SerializedGraphEdge = {
    from: string,
    to: string,
    weight?: number
}
"#;
impl GraphEdge {
    pub fn new(from: String, to: String, weight: Option<f64>) -> Self {
        Self { from, to, weight }
    }
}
impl fmt::Display for GraphEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.weight {
            Some(w) => format!("{}:{}", self.to, w),
            None => self.to.clone(),
        };
        write!(f, "{}", s)
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    name: String,
    edges: HashMap<String, GraphEdge>,
}
#[wasm_bindgen(typescript_custom_section)]
const IGraphNode: &'static str = r#"
export type SerializedGraphNode = {
    name: string,
    edges: { [key: string]: SerializedGraphEdge }
}
"#;
impl GraphNode {
    pub fn new(name: String, edges: Vec<GraphEdge>) -> Self {
        let edges = edges
            .into_iter()
            .map(|edge| (edge.to.clone(), edge))
            .collect::<HashMap<String, GraphEdge>>();
        Self { name, edges }
    }
    pub fn to_edges(self) -> Vec<GraphEdge> {
        self.edges.into_values().collect()
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
}
impl fmt::Display for GraphNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let edges = self
            .edges
            .values()
            .map(|edge| edge.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        if edges.is_empty() {
            return f.write_str(&self.name);
        }
        write!(f, "{} -> [ {} ]", self.name, edges)
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Graph {
    vertices: Vec<GraphNode>,
}
#[wasm_bindgen(typescript_custom_section)]
const IGraph: &'static str = r#"
export type SerializedGraph = {
    vertices: SerializedGraphNode[]
}
"#;
impl Graph {
    pub fn new(vertices: Vec<GraphNode>) -> Self {
        Self { vertices }
    }
    pub fn to_edges(self) -> Vec<GraphEdge> {
        self.vertices
            .into_iter()
            .flat_map(|node| node.edges.into_values().collect::<Vec<_>>())
            .collect::<Vec<_>>()
    }
    pub fn nodes(&self) -> &Vec<GraphNode> {
        &self.vertices
    }

    pub fn to_nodes(self) -> Vec<GraphNode> {
        self.vertices
    }
    pub fn vertices(&self) -> &Vec<GraphNode> {
        &self.vertices
    }
    pub fn neighbour_of(&self, node_name: &str) -> Result<Vec<&GraphEdge>, TransformError> {
        let node = self
            .vertices
            .iter()
            .find(|n: &&GraphNode| n.name == node_name);
        match node {
            Some(node) => Ok(node.edges.values().collect()),
            None => Err(TransformError::Other(format!(
                "node {} not found in graph",
                node_name
            ))),
        }
    }
    pub fn into_neighbours_of(self, node_name: &str) -> Result<Vec<GraphEdge>, TransformError> {
        let node = self
            .vertices
            .into_iter()
            .find(|n: &GraphNode| n.name == node_name);
        match node {
            Some(node) => Ok(node.edges.into_values().collect()),
            None => Err(TransformError::Other(format!(
                "node {} not found in graph",
                node_name
            ))),
        }
    }
}
impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes = self
            .vertices
            .iter()
            .map(|node| format!("    {}", node.to_string()))
            .collect::<Vec<_>>()
            .join(",\n");
        if nodes.is_empty() {
            return write!(f, "Graph {{ }}");
        }
        write!(f, "Graph {{\n{}\n}}", nodes)
    }
}

impl ApplyOp for GraphNode {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(
            op,
            PrimitiveKind::GraphNode,
        ))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            PrimitiveKind::GraphNode,
        ))
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        false
    }
}
impl ApplyOp for GraphEdge {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(
            op,
            PrimitiveKind::GraphEdge,
        ))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            PrimitiveKind::GraphEdge,
        ))
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        false
    }
}
impl ApplyOp for Graph {
    type TargetType = PrimitiveKind;
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(
            op,
            PrimitiveKind::Graph,
        ))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            PrimitiveKind::Graph,
        ))
    }
    fn can_apply_binary_op(op: BinOp, to: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(op: UnOp) -> bool {
        false
    }
}

impl Spreadable for GraphEdge {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Ok(vec![
            Primitive::String(self.from),
            Primitive::Number(self.weight.unwrap_or(1.0)),
            Primitive::String(self.to),
        ])
    }
}
impl Spreadable for GraphNode {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::GraphNode))
    }
}
impl Spreadable for Graph {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Err(TransformError::Unspreadable(PrimitiveKind::Graph))
    }
}
