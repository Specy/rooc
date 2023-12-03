use std::collections::HashMap;

use crate::parser::transformer::TransformError;

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
    pub fn edges(&self) -> Vec<GraphEdge> {
        self.vertices
            .iter()
            .map(|node| {
                node.edges
                    .values()
                    .map(|edge| edge.clone())
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
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
