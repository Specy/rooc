use crate::{primitives::primitive::Primitive, utils::Spanned};

use super::{transformer::{TransformerContext, VariableType, TransformError}, pre_parsed_problem::IterableSet};

//TODO make this a iterator
pub fn recursive_set_resolver<T>(
    sets: &Vec<IterableSet>,
    context: &mut TransformerContext,
    results: &mut Vec<T>,
    current_level: usize,
    on_leaf: &dyn Fn(&mut TransformerContext) -> Result<T, TransformError>,
) -> Result<(), TransformError> {
    let range = sets.get(current_level).unwrap();
    context.add_scope();
    match &range.var {
        VariableType::Single(n) => {
            context
                .declare_variable(n, Primitive::Undefined, true)
                .map_err(|e| e.to_spanned_error(&range.span))?;
        }
        VariableType::Tuple(t) => {
            for name in t.iter() {
                context
                    .declare_variable(name, Primitive::Undefined, true)
                    .map_err(|e| e.to_spanned_error(&range.span))?;
            }
        }
    }
    let values = range.iterator.as_iterator(&context)?;
    let values = values.to_primitive_set();
    for value in values.into_iter() {
        match &range.var {
            VariableType::Single(n) => {
                context
                    .update_variable(n, value.clone())
                    .map_err(|e| e.to_spanned_error(&range.span))?;
            }
            VariableType::Tuple(tuple) => {
                match value {
                    Primitive::Tuple(v) => apply_tuple(context, tuple, v.into_primitives())
                        .map_err(|e| e.to_spanned_error(&range.span))?,
                    Primitive::GraphEdge(e) => {
                        let v = vec![
                            Primitive::String(e.from.clone()), //TODO maybe i should return the actul edge instead
                            Primitive::Number(e.weight.unwrap_or(1.0)),
                            Primitive::String(e.to.clone()),
                        ];
                        apply_tuple(context, tuple, v)
                            .map_err(|e| e.to_spanned_error(&range.span))?
                    }
                    _ => {
                        return Err(TransformError::WrongArgument(format!(
                            "Expected spreadable primitive, got {}",
                            value.get_type().to_string()
                        )))
                    }
                }
            }
        }
        if current_level + 1 >= sets.len() {
            let value = on_leaf(context)?;
            results.push(value); //TODO should i do this? maybe it's best to leave it out to the caller
        } else {
            recursive_set_resolver(sets, context, results, current_level + 1, on_leaf)
                .map_err(|e| e.to_spanned_error(&range.span))?;
        }
    }
    context.pop_scope()?;
    Ok(())
}



pub fn apply_tuple(
    context: &mut TransformerContext,
    tuple: &Vec<Spanned<String>>,
    spreadable: Vec<Primitive>,
) -> Result<(), TransformError> {
    if tuple.len() > spreadable.len() {
        return Err(TransformError::WrongArgument(format!(
            "Cannot destructure tuple of length {} in {} elements",
            spreadable.len(),
            tuple.len()
        )));
    }
    for (i, value) in spreadable.into_iter().enumerate() {
        let name = tuple.get(i);
        match name {
            Some(name) => {
                context
                    .update_variable(name, value)
                    .map_err(|e| e.to_spanned_error(&name.get_span()))?;
            }
            None => return Ok(()), //tuple is smaller than the spreadable, ignore the rest
        }
    }
    Ok(())
}
