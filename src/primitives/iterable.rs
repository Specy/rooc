use crate::{
    check_bounds,
    parser::{parser::ArrayAccess, transformer::TransformError},
};

use super::{
    graph::{Graph, GraphEdge, GraphNode},
    primitive::{Primitive, Tuple},
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
    //TODO make this a macro
    pub fn to_string(&self) -> String {
        match self {
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
    pub fn to_primitive_set(self) -> Vec<Primitive> {
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
            IterableKind::Nodes(v) => v
                .into_iter()
                .map(|n| Primitive::GraphNode(n.to_owned()))
                .collect(),
            IterableKind::Tuple(v) => v.into_iter().map(|t| Primitive::Tuple(t)).collect(),
            IterableKind::Iterable(v) => v.into_iter().map(|i| Primitive::Iterable(i)).collect(),
            IterableKind::Booleans(v) => v.into_iter().map(|b| Primitive::Boolean(b)).collect(),
            IterableKind::Graphs(v) => v.into_iter().map(|g| Primitive::Graph(g)).collect(),
        }
    }

    //TODO refactor this
    pub fn read(&self, indexes: Vec<usize>) -> Result<Primitive, TransformError> {
        if indexes.len() == 0 {
            return Ok(Primitive::Undefined);
        }

        let mut current = self;
        let mut indexes = indexes;
        while indexes.len() > 0 {
            let i = indexes.remove(0);
            let ended = indexes.len() == 0;
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
                                self.to_string()
                            )));
                        }
                    }
                    _ => {
                        return Err(TransformError::OutOfBounds(format!(
                            "cannot access index {} of {}",
                            i,
                            self.to_string()
                        )));
                    }
                }
            }
        }
        Err(TransformError::OutOfBounds(format!(
            "cannot access index {} of {}",
            indexes[0],
            self.to_string()
        )))
    }

}
