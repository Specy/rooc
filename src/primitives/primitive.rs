use core::fmt;

use serde::Serialize;

use crate::{
    bail_wrong_argument, match_or_bail, parser::transformer::TransformError, wrong_argument,
};

use super::{
    graph::{Graph, GraphEdge, GraphNode},
    iterable::IterableKind,
    tuple::Tuple,
};

#[derive(Debug, Clone, Serialize)]
pub enum Primitive {
    Number(f64),
    String(String),
    //TODO instead of making these, make a recursive IterableKind
    Iterable(IterableKind),
    Graph(Graph),
    GraphEdge(GraphEdge),
    GraphNode(GraphNode),
    Tuple(Tuple),
    Boolean(bool),
    Undefined,
}

#[derive(Debug, Clone, Serialize)]
pub enum PrimitiveKind {
    Number,
    String,
    Iterable,
    Graph,
    GraphEdge,
    GraphNode,
    Tuple,
    Boolean,
    Undefined,
}
impl PrimitiveKind {
    pub fn from_primitive(p: &Primitive) -> Self {
        match p {
            Primitive::Number(_) => PrimitiveKind::Number,
            Primitive::String(_) => PrimitiveKind::String,
            Primitive::Iterable(_) => PrimitiveKind::Iterable,
            Primitive::Graph(_) => PrimitiveKind::Graph,
            Primitive::GraphEdge(_) => PrimitiveKind::GraphEdge,
            Primitive::GraphNode(_) => PrimitiveKind::GraphNode,
            Primitive::Tuple(_) => PrimitiveKind::Tuple,
            Primitive::Boolean(_) => PrimitiveKind::Boolean,
            Primitive::Undefined => PrimitiveKind::Undefined,
        }
    }
    pub fn to_string(&self) -> &'static str {
        match self {
            PrimitiveKind::Number => "Number",
            PrimitiveKind::String => "String",
            PrimitiveKind::Iterable => "Iterable",
            PrimitiveKind::Graph => "Graph",
            PrimitiveKind::GraphEdge => "GraphEdge",
            PrimitiveKind::GraphNode => "GraphNode",
            PrimitiveKind::Tuple => "Tuple",
            PrimitiveKind::Boolean => "Boolean",
            PrimitiveKind::Undefined => "Undefined",
        }
    }
}

impl Primitive {
    pub fn get_type(&self) -> PrimitiveKind {
        PrimitiveKind::from_primitive(self)
    }
    pub fn get_type_string(&self) -> &'static str {
        self.get_type().to_string()
    }
    pub fn as_number(&self) -> Result<f64, TransformError> {
        match_or_bail!("Number", Primitive::Number(n) => Ok(*n) ; (self, self))
    }
    pub fn as_integer(&self) -> Result<i64, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("Integer", self)
        } else {
            Ok(n as i64)
        }
    }
    pub fn as_usize(&self) -> Result<usize, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("Integer", self)
        } else if n < 0.0 {
            bail_wrong_argument!("Positive integer", self)
        } else {
            Ok(n as usize)
        }
    }
    pub fn as_graph(&self) -> Result<&Graph, TransformError> {
        match_or_bail!("Graph", 
            Primitive::Graph(g) => Ok(g)
          ; (self, self))
    }
    pub fn as_iterator(&self) -> Result<&IterableKind, TransformError> {
        match_or_bail!("Iterable", Primitive::Iterable(i) => Ok(i) ; (self, self))
    }
    pub fn as_tuple(&self) -> Result<&Vec<Primitive>, TransformError> {
        match_or_bail!("Tuple", Primitive::Tuple(t) => Ok(t.get_primitives()) ; (self, self))
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Primitive::Number(n) => n.to_string(),
            Primitive::String(s) => s.to_string(),
            Primitive::Iterable(i) => match i {
                IterableKind::Numbers(v) => format!("{:?}", v),
                IterableKind::Strings(v) => format!("{:?}", v),
                IterableKind::Edges(v) => format!("{:?}", v),
                IterableKind::Nodes(v) => format!("{:?}", v),
                IterableKind::Tuple(v) => format!("{:?}", v),
                IterableKind::Booleans(v) => format!("{:?}", v),
                IterableKind::Graphs(v) => format!("{:?}", v),
                IterableKind::Iterable(v) => {
                    let result = v
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("[{}]", result)
                }
            },
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
