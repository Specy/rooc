use std::collections::HashMap;

use crate::{
    math::operators::{BinOp, UnOp},
    parser::transformer::TransformError,
};

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub weight: Option<f64>,
}
impl GraphEdge {
    pub fn new(from: String, to: String, weight: Option<f64>) -> Self {
        Self { from, to, weight }
    }
    pub fn to_string(&self) -> String {
        match self.weight {
            Some(w) => format!("{}:{}", self.to, w),
            None => self.to.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    name: String,
    edges: HashMap<String, GraphEdge>,
}
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
    pub fn to_string(&self) -> String {
        let edges = self
            .edges
            .iter()
            .map(|(_, edge)| edge.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}: {{{}}}", self.name, edges)
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    vertices: Vec<GraphNode>,
}
impl Graph {
    pub fn new(vertices: Vec<GraphNode>) -> Self {
        Self { vertices }
    }
    pub fn to_edges(self) -> Vec<GraphEdge> {
        self.vertices
            .into_iter()
            .map(|node| node.edges.into_values().collect::<Vec<_>>())
            .flatten()
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
            None => {
                return Err(TransformError::Other(format!(
                    "node {} not found in graph",
                    node_name
                )))
            }
        }
    }
    pub fn into_neighbours_of(self, node_name: &str) -> Result<Vec<GraphEdge>, TransformError> {
        let node = self
            .vertices
            .into_iter()
            .find(|n: &GraphNode| n.name == node_name);
        match node {
            Some(node) => Ok(node.edges.into_values().collect()),
            None => {
                return Err(TransformError::Other(format!(
                    "node {} not found in graph",
                    node_name
                )))
            }
        }
    }
    pub fn to_string(&self) -> String {
        let nodes = self
            .vertices
            .iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        format!("[{}]", nodes)
    }
}

impl ApplyOp for GraphNode {
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
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
}
impl ApplyOp for GraphEdge {
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
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
}
impl ApplyOp for Graph {
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, to: &Primitive) -> Result<Primitive, OperatorError> {
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
