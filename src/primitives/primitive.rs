use crate::{
    bail_wrong_argument, match_or_bail, parser::transformer::TransformError, wrong_argument,
};

use super::{
    consts::ConstantValue,
    graph::{Graph, GraphEdge, GraphNode},
    iterable::IterableKind,
};

#[derive(Debug, Clone)]
pub enum Primitive {
    Number(f64),
    String(String),
    //TODO instead of making these, make a recursive IterableKind
    Iterable(IterableKind),
    Graph(Graph),
    GraphEdge(GraphEdge),
    GraphNode(GraphNode),
    Tuple(Vec<Primitive>),
    Undefined,
}

impl Primitive {
    pub fn from_constant_value(value: ConstantValue) -> Self {
        match value {
            ConstantValue::Number(n) => Primitive::Number(n),
            ConstantValue::OneDimArray(v) => Primitive::Iterable(IterableKind::Numbers(v)),
            ConstantValue::TwoDimArray(v) => {
                let inner = v
                    .into_iter()
                    .map(|row| IterableKind::Numbers(row))
                    .collect::<Vec<_>>();
                Primitive::Iterable(IterableKind::Iterable(inner))
            }
            ConstantValue::Graph(g) => Primitive::Graph(g),
            ConstantValue::String(s) => Primitive::String(s),
        }
    }
    pub fn as_number(&self) -> Result<f64, TransformError> {
        match_or_bail!("number", Primitive::Number(n) => Ok(*n) ; (self, self))
    }
    pub fn as_integer(&self) -> Result<i64, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("integer", self)
        } else {
            Ok(n as i64)
        }
    }
    pub fn as_usize(&self) -> Result<usize, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("integer", self)
        } else if n < 0.0 {
            bail_wrong_argument!("positive integer", self)
        } else {
            Ok(n as usize)
        }
    }
    pub fn as_graph(&self) -> Result<&Graph, TransformError> {
        match_or_bail!("graph", 
            Primitive::Graph(g) => Ok(g)
          ; (self, self))
    }
    pub fn as_number_array(&self) -> Result<&Vec<f64>, TransformError> {
        match self {
            Primitive::Iterable(IterableKind::Numbers(a)) => Ok(a),
            _ => bail_wrong_argument!("array1d", self),
        }
    }
    pub fn as_number_matrix(&self) -> Result<Vec<&Vec<f64>>, TransformError> {
        match self {
            Primitive::Iterable(IterableKind::Iterable(a)) => a
                .into_iter()
                .map(|row| match row {
                    IterableKind::Numbers(v) => Ok(v),
                    _ => bail_wrong_argument!("array2d", self),
                })
                .collect::<Result<Vec<_>, _>>(),
            _ => bail_wrong_argument!("array2d", self),
        }
    }
    pub fn as_iterator(&self) -> Result<&IterableKind, TransformError> {
        match_or_bail!("iterable", Primitive::Iterable(i) => Ok(i) ; (self, self))
    }
    pub fn as_tuple(&self) -> Result<&Vec<Primitive>, TransformError> {
        match_or_bail!("tuple", Primitive::Tuple(t) => Ok(t) ; (self, self))
    }

    pub fn to_string(&self) -> String {
        //TODO improve this
        match self {
            Primitive::Number(n) => n.to_string(),
            Primitive::String(s) => s.to_string(),
            Primitive::Iterable(i) => match i {
                IterableKind::Numbers(v) => format!("{:?}", v),
                IterableKind::Strings(v) => format!("{:?}", v),
                IterableKind::Edges(v) => format!("{:?}", v),
                IterableKind::Nodes(v) => format!("{:?}", v),
                IterableKind::Tuple(v) => format!("{:?}", v),
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
            Primitive::Undefined => "undefined".to_string(),
        }
    }
}
