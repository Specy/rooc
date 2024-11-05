use crate::primitives::{IterableKind, Primitive, PrimitiveKind};

/// Flattens an array of primitives into a single primitive iterable if possible.
/// Alternatively returns a mixed value array
///
/// # Arguments
/// * `values` - Vector of primitives to flatten
///
/// # Returns
/// A single Primitive containing an IterableKind with the flattened values
pub fn flatten_primitive_array_values(values: Vec<Primitive>) -> Primitive {
    let first = values.first();
    if first.is_none() {
        return Primitive::Iterable(IterableKind::Anys(vec![]));
    }
    let first_kind = first.unwrap().get_type();
    let all_equal_type = values.iter().all(|v| v.get_type() == first_kind);
    if !all_equal_type {
        return Primitive::Iterable(IterableKind::Anys(values));
    }

    //TODO try to make this return a Mixed Primitive if the types are different, instead of failing
    match first_kind {
        PrimitiveKind::Any => Primitive::Iterable(IterableKind::Numbers(vec![])), //can never happen
        PrimitiveKind::Boolean => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Boolean(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Booleans(values))
        }
        PrimitiveKind::Number => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Number(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Numbers(values))
        }
        PrimitiveKind::Integer => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Integer(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Integers(values))
        }
        PrimitiveKind::PositiveInteger => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::PositiveInteger(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::PositiveIntegers(values))
        }
        PrimitiveKind::String => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::String(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Strings(values))
        }
        PrimitiveKind::GraphEdge => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::GraphEdge(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Edges(values))
        }
        PrimitiveKind::GraphNode => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::GraphNode(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Nodes(values))
        }
        PrimitiveKind::Tuple(_) => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Tuple(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Tuples(values))
        }
        PrimitiveKind::Iterable(_) => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Iterable(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Iterables(values))
        }
        PrimitiveKind::Undefined => Primitive::Iterable(IterableKind::Numbers(vec![])),
        PrimitiveKind::Graph => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Graph(b) => b,
                    _ => unreachable!(),
                })
                .collect();
            Primitive::Iterable(IterableKind::Graphs(values))
        }
    }
}
