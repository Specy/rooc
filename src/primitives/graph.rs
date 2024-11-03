use core::fmt;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use crate::prelude::*;

use crate::math::{BinOp, UnOp};
use crate::parser::model_transformer::TransformError;
use crate::traits::{escape_latex, ToLatex};

use super::{
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub weight: Option<f64>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
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

impl ToLatex for GraphEdge {
    fn to_latex(&self) -> String {
        if let Some(w) = self.weight {
            format!("\\text{{{}:{}}}", escape_latex(&self.to), w)
        } else {
            format!("\\text{{{}}}", escape_latex(&self.to))
        }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    name: String,
    edges: IndexMap<String, GraphEdge>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
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
            .collect::<IndexMap<String, GraphEdge>>();
        Self { name, edges }
    }
    pub fn to_edges(self) -> Vec<GraphEdge> {
        self.edges.into_values().collect()
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl ToLatex for GraphNode {
    fn to_latex(&self) -> String {
        let edges = self
            .edges
            .values()
            .map(|edge| edge.to_latex())
            .collect::<Vec<_>>()
            .join(",\\ ");
        if edges.is_empty() {
            self.name.clone()
        } else {
            format!("{}\\to\\left\\{{{}\\right\\}}", self.name, edges)
        }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    vertices: Vec<GraphNode>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
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

//TODO decide if this is a nice enough representation
impl ToLatex for Graph {
    fn to_latex(&self) -> String {
        let nodes = self
            .vertices
            .iter()
            .map(|node| node.to_latex())
            .collect::<Vec<_>>()
            .join("\\\\ ");
        if nodes.is_empty() {
            return "\\emptyset".to_string();
        }
        format!("\\begin{{Bmatrix*}}[l] {} \\end{{Bmatrix*}}", nodes)
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes = self
            .vertices
            .iter()
            .map(|node| format!("    {}", node))
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
    fn can_apply_binary_op(_: BinOp, _: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(_: UnOp) -> bool {
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
    fn can_apply_binary_op(_: BinOp, _: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(_: UnOp) -> bool {
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
    fn can_apply_binary_op(_: BinOp, _: Self::TargetType) -> bool {
        false
    }
    fn can_apply_unary_op(_: UnOp) -> bool {
        false
    }
}

impl Spreadable for GraphEdge {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Ok(vec![
            Primitive::String(self.from),
            Primitive::String(self.to),
            Primitive::Number(self.weight.unwrap_or(1.0)),
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
