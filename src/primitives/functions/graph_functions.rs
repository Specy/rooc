use std::fmt::Debug;

use pest::iterators::Pair;
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    bail_incorrect_type_signature, bail_incorrect_type_signature_of_fn,
    bail_wrong_number_of_arguments,
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformerContext, TransformError},
    },
    primitives::{
        iterable::IterableKind,
        primitive::{Primitive, PrimitiveKind},
    },
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext, WithType},
    utils::InputSpan
    ,
};

use super::function_traits::FunctionCall;

#[derive(Debug, Serialize, Clone)]
pub struct EdgesOfGraphFn {
    args: Vec<PreExp>,
    span: InputSpan,
}

impl WithType for EdgesOfGraphFn {
    fn get_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }
}

impl TypeCheckable for EdgesOfGraphFn {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self.args[..] {
            [ref of_graph] => {
                if !matches!(of_graph.get_type(context), PrimitiveKind::Graph) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Graph,
                        of_graph.get_type(context),
                        of_graph.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => bail_incorrect_type_signature_of_fn!(self, context),
        }
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.args
            .iter()
            .for_each(|arg| arg.populate_token_type_map(context));
    }
}

impl FunctionCall for EdgesOfGraphFn {
    fn from_parameters(args: Vec<PreExp>, rule: &Pair<Rule>) -> Self {
        Self {
            args,
            span: InputSpan::from_pair(rule),
        }
    }
    fn get_span(&self) -> &InputSpan {
        &self.span
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self.args[..] {
            [ref of_graph] => {
                let graph = of_graph.as_graph(context)?;
                let edges = graph.to_edges();
                Ok(Primitive::Iterable(IterableKind::Edges(edges)))
            }
            _ => bail_wrong_number_of_arguments!(self),
        }
    }
    fn to_string(&self) -> String {
        format!(
            "{}({})",
            self.get_function_name(),
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
    fn get_function_name(&self) -> String {
        "edges".to_string()
    }
    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![("of_graph".to_string(), PrimitiveKind::Graph)]
    }
    fn get_parameters(&self) -> &Vec<PreExp> {
        &self.args
    }
}

#[wasm_bindgen(typescript_custom_section)]
#[derive(Debug, Serialize, Clone)]
pub struct NodesOfGraphFn {
    args: Vec<PreExp>,
    span: InputSpan,
}

impl WithType for NodesOfGraphFn {
    fn get_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphNode))
    }
}

impl TypeCheckable for NodesOfGraphFn {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self.args[..] {
            [ref of_graph] => {
                if !matches!(of_graph.get_type(context), PrimitiveKind::Graph) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Graph,
                        of_graph.get_type(context),
                        of_graph.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => bail_incorrect_type_signature_of_fn!(self, context),
        }
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.args
            .iter()
            .for_each(|arg| arg.populate_token_type_map(context));
    }
}

impl FunctionCall for NodesOfGraphFn {
    fn from_parameters(args: Vec<PreExp>, rule: &Pair<Rule>) -> Self {
        Self {
            args,
            span: InputSpan::from_pair(rule),
        }
    }
    fn get_span(&self) -> &InputSpan {
        &self.span
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self.args[..] {
            [ref of_graph] => {
                let graph = of_graph.as_graph(context)?;
                let nodes = graph.to_nodes();
                Ok(Primitive::Iterable(IterableKind::Nodes(nodes)))
            }
            _ => bail_wrong_number_of_arguments!(self),
        }
    }
    fn to_string(&self) -> String {
        format!(
            "{}({})",
            self.get_function_name(),
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
    fn get_function_name(&self) -> String {
        "nodes".to_string()
    }

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![("of_graph".to_string(), PrimitiveKind::Graph)]
    }
    fn get_parameters(&self) -> &Vec<PreExp> {
        &self.args
    }
}


#[derive(Debug, Serialize, Clone)]
pub struct NeighbourOfNodeFn {
    args: Vec<PreExp>,
    span: InputSpan,
}

impl TypeCheckable for NeighbourOfNodeFn {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self.args[..] {
            [ref of_node] => {
                if !matches!(of_node.get_type(context), PrimitiveKind::GraphNode) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::GraphNode,
                        of_node.get_type(context),
                        of_node.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => bail_incorrect_type_signature_of_fn!(self, context),
        }
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.args
            .iter()
            .for_each(|arg| arg.populate_token_type_map(context));
    }
}

impl WithType for NeighbourOfNodeFn {
    fn get_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }
}

impl FunctionCall for NeighbourOfNodeFn {
    fn from_parameters(args: Vec<PreExp>, rule: &Pair<Rule>) -> Self {
        Self {
            args,
            span: InputSpan::from_pair(rule),
        }
    }
    fn get_span(&self) -> &InputSpan {
        &self.span
    }

    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self.args[..] {
            [ref of_node] => {
                let node = of_node.as_node(context)?;
                Ok(Primitive::Iterable(IterableKind::Edges(node.to_edges())))
            }
            _ => bail_wrong_number_of_arguments!(self),
        }
    }

    fn to_string(&self) -> String {
        format!(
            "{}({})",
            self.get_function_name(),
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
    fn get_function_name(&self) -> String {
        "neigh_edges".to_string()
    }
    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![
            ("of_node".to_string(), PrimitiveKind::GraphNode),
        ]
    }
    fn get_parameters(&self) -> &Vec<PreExp> {
        &self.args
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NeighboursOfNodeInGraphFn {
    args: Vec<PreExp>,
    span: InputSpan,
}

impl WithType for NeighboursOfNodeInGraphFn {
    fn get_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }
}

impl TypeCheckable for NeighboursOfNodeInGraphFn {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        match self.args[..] {
            [ref of_node, ref in_graph] => {
                if !matches!(of_node.get_type(context), PrimitiveKind::String) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::String,
                        of_node.get_type(context),
                        of_node.get_span().clone(),
                    ))
                } else if !matches!(in_graph.get_type(context), PrimitiveKind::Graph) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Graph,
                        in_graph.get_type(context),
                        in_graph.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => bail_incorrect_type_signature_of_fn!(self, context),
        }
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.args
            .iter()
            .for_each(|arg| arg.populate_token_type_map(context));
    }
}

impl FunctionCall for NeighboursOfNodeInGraphFn {
    fn from_parameters(args: Vec<PreExp>, rule: &Pair<Rule>) -> Self {
        Self {
            args,
            span: InputSpan::from_pair(rule),
        }
    }
    fn get_span(&self) -> &InputSpan {
        &self.span
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self.args[..] {
            [ref of_node, ref in_graph] => {
                let node = of_node.as_string(context)?;
                let graph = in_graph.as_graph(context)?;
                let neighbours = graph.into_neighbours_of(&node)?;
                Ok(Primitive::Iterable(IterableKind::Edges(neighbours)))
            }
            _ => bail_wrong_number_of_arguments!(self),
        }
    }
    fn to_string(&self) -> String {
        format!(
            "{}({})",
            self.get_function_name(),
            self.args
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![
            ("of_node_name".to_string(), PrimitiveKind::String),
            ("in_graph".to_string(), PrimitiveKind::Graph),
        ]
    }
    fn get_function_name(&self) -> String {
        "neigh_edges_of".to_string()
    }
    fn get_parameters(&self) -> &Vec<PreExp> {
        &self.args
    }
}

#[wasm_bindgen(typescript_custom_section)]
const FN_neigh_edges_of: &'static str = r#"

"#;