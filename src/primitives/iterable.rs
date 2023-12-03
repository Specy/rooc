use super::{
    graph::{GraphEdge, GraphNode},
    primitive::Primitive,
};

#[derive(Debug, Clone)]
pub enum IterableKind {
    Numbers(Vec<f64>),
    Strings(Vec<String>),
    Edges(Vec<GraphEdge>),
    Nodes(Vec<GraphNode>),
    Tuple(Vec<Vec<Primitive>>),
    Iterable(Vec<IterableKind>),
}
impl IterableKind {
    pub fn to_string(&self) -> String {
        match self {
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
        }
    }
}
