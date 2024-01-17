use crate::primitives::{
    iterable::IterableKind,
    primitive::{Primitive, PrimitiveKind},
};

//TODO make this a macro
pub fn flatten_primitive_array_values(values: Vec<Primitive>) -> Result<Primitive, String> {
    let first = values.first();
    if first.is_none() {
        return Ok(Primitive::Iterable(IterableKind::Numbers(vec![])));
    }
    let first_kind = first.unwrap().get_type();
    match first_kind {
        PrimitiveKind::Any => Ok(Primitive::Iterable(IterableKind::Numbers(vec![]))), //can never happen
        PrimitiveKind::Boolean => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Boolean(b) => Ok(b),
                    _ => Err(format!(
                        "Expected Boolean but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Booleans(values)))
        }
        PrimitiveKind::Number => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Number(n) => Ok(n),
                    _ => Err(format!(
                        "Expected Number, got \"{}\"",
                        v.get_type_string(),
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Numbers(values)))
        }
        PrimitiveKind::Integer => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Integer(i) => Ok(i),
                    _ => Err(format!(
                        "Expected Integer but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Integers(values)))
        }
        PrimitiveKind::PositiveInteger => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::PositiveInteger(i) => Ok(i),
                    _ => Err(format!(
                        "Expected PositiveInteger but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::PositiveIntegers(values)))
        }
        PrimitiveKind::String => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::String(s) => Ok(s),
                    _ => Err(format!(
                        "Expected String but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Strings(values)))
        }
        PrimitiveKind::GraphEdge => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::GraphEdge(e) => Ok(e),
                    _ => Err(format!(
                        "Expected GraphEdge but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Edges(values)))
        }
        PrimitiveKind::GraphNode => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::GraphNode(n) => Ok(n),
                    _ => Err(format!(
                        "Expected GraphNode but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Nodes(values)))
        }
        PrimitiveKind::Tuple(_) => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Tuple(t) => Ok(t),
                    _ => Err(format!(
                        "Expected Tuple but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Tuple(values)))
        }
        PrimitiveKind::Iterable(_) => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Iterable(i) => Ok(i),
                    _ => Err(format!(
                        "Expected Iterable but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Iterable(values)))
        }
        PrimitiveKind::Undefined => Ok(Primitive::Iterable(IterableKind::Numbers(vec![]))),
        PrimitiveKind::Graph => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Graph(g) => Ok(g),
                    _ => Err(format!(
                        "Expected Graph but got {}",
                        v.get_type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Graphs(values)))
        }
    }
}
