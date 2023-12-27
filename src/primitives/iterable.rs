use core::fmt;

use crate::{
    check_bounds,
    math::operators::{BinOp, UnOp},
    parser::transformer::TransformError,
};

use super::{
    graph::{Graph, GraphEdge, GraphNode},
    primitive::{Primitive, PrimitiveKind},
    primitive_traits::{ApplyOp, OperatorError, Spreadable},
    tuple::Tuple,
};

#[derive(Debug, Clone)]
pub enum IterableKind {
    Numbers(Vec<f64>),
    Strings(Vec<String>),
    Edges(Vec<GraphEdge>),
    Nodes(Vec<GraphNode>),
    Graphs(Vec<Graph>),
    Tuple(Vec<Tuple>),
    Booleans(Vec<bool>),
    Iterable(Vec<IterableKind>),
}
impl IterableKind {
    pub fn get_argument_name(&self) -> &'static str {
        match self {
            IterableKind::Numbers(_) => "number",
            IterableKind::Strings(_) => "string",
            IterableKind::Edges(_) => "edge",
            IterableKind::Nodes(_) => "node",
            IterableKind::Tuple(_) => "tuple",
            IterableKind::Booleans(_) => "boolean",
            IterableKind::Graphs(_) => "graph",
            IterableKind::Iterable(_) => "iterable",
        }
    }
    pub fn len(&self) -> usize {
        match self {
            IterableKind::Numbers(v) => v.len(),
            IterableKind::Strings(v) => v.len(),
            IterableKind::Edges(v) => v.len(),
            IterableKind::Nodes(v) => v.len(),
            IterableKind::Tuple(v) => v.len(),
            IterableKind::Iterable(v) => v.len(),
            IterableKind::Booleans(v) => v.len(),
            IterableKind::Graphs(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            IterableKind::Numbers(v) => v.is_empty(),
            IterableKind::Strings(v) => v.is_empty(),
            IterableKind::Edges(v) => v.is_empty(),
            IterableKind::Nodes(v) => v.is_empty(),
            IterableKind::Tuple(v) => v.is_empty(),
            IterableKind::Iterable(v) => v.is_empty(),
            IterableKind::Booleans(v) => v.is_empty(),
            IterableKind::Graphs(v) => v.is_empty(),
        }
    }
    pub fn to_primitives(self) -> Vec<Primitive> {
        match self {
            IterableKind::Numbers(v) => v.iter().map(|n| Primitive::Number(*n)).collect(),
            IterableKind::Strings(v) => v
                .into_iter()
                .map(|s| Primitive::String((*s).to_string()))
                .collect(),
            IterableKind::Edges(v) => v
                .iter()
                .map(|e| Primitive::GraphEdge(e.to_owned()))
                .collect(),
            IterableKind::Nodes(v) => v.into_iter().map(Primitive::GraphNode).collect(),
            IterableKind::Tuple(v) => v.into_iter().map(Primitive::Tuple).collect(),
            IterableKind::Iterable(v) => v.into_iter().map(Primitive::Iterable).collect(),
            IterableKind::Booleans(v) => v.into_iter().map(Primitive::Boolean).collect(),
            IterableKind::Graphs(v) => v.into_iter().map(Primitive::Graph).collect(),
        }
    }

    //TODO refactor this
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
                    IterableKind::Numbers(v) => check_bounds!(i, v, self, Primitive::Number(v[i])),
                    IterableKind::Strings(v) => {
                        check_bounds!(i, v, self, Primitive::String(v[i].to_string()))
                    }
                    IterableKind::Edges(v) => {
                        check_bounds!(i, v, self, Primitive::GraphEdge(v[i].to_owned()))
                    }
                    IterableKind::Nodes(v) => {
                        check_bounds!(i, v, self, Primitive::GraphNode(v[i].to_owned()))
                    }
                    IterableKind::Tuple(v) => {
                        check_bounds!(i, v, self, Primitive::Tuple(v[i].clone()))
                    }
                    IterableKind::Iterable(v) => {
                        check_bounds!(i, v, self, Primitive::Iterable(v[i].clone()))
                    }
                    IterableKind::Graphs(v) => {
                        check_bounds!(i, v, self, Primitive::Graph(v[i].clone()))
                    }
                };
                return Ok(val);
            } else {
                match current {
                    IterableKind::Iterable(v) => {
                        if i < v.len() {
                            current = &v[i];
                        } else {
                            return Err(TransformError::OutOfBounds(format!(
                                "cannot access index {} of {}",
                                i,
                                self
                            )));
                        }
                    }
                    _ => {
                        return Err(TransformError::OutOfBounds(format!(
                            "cannot access index {} of {}",
                            i,
                            self
                        )));
                    }
                }
            }
        }
        Err(TransformError::OutOfBounds(format!(
            "cannot access index {} of {}",
            indexes[0],
            self
        )))
    }
}
impl fmt::Display for IterableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IterableKind::Edges(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            IterableKind::Nodes(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            IterableKind::Numbers(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            IterableKind::Strings(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            IterableKind::Tuple(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| {
                            format!(
                                "[{}]",
                                e.get_primitives()
                                    .iter()
                                    .map(|e| e.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",\n")
                )
            }
            IterableKind::Iterable(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            IterableKind::Booleans(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            IterableKind::Graphs(v) => {
                format!(
                    "[{}]",
                    v.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        };
        f.write_str(&s)
    }
}

impl ApplyOp for IterableKind {
    type Target = Primitive;
    type Error = OperatorError;
    fn apply_binary_op(&self, op: BinOp, _to: &Primitive) -> Result<Primitive, OperatorError> {
        Err(OperatorError::unsupported_bin_operation(
            op,
            PrimitiveKind::Iterable,
        ))
    }
    fn apply_unary_op(&self, op: UnOp) -> Result<Self::Target, Self::Error> {
        Err(OperatorError::unsupported_un_operation(
            op,
            PrimitiveKind::Iterable,
        ))
    }
}
impl Spreadable for IterableKind {
    fn to_primitive_set(self) -> Result<Vec<Primitive>, TransformError> {
        Ok(self.to_primitives())
    }
}
