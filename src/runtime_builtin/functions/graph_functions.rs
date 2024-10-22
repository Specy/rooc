use std::fmt::Debug;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use super::function_traits::{
    default_type_check, default_wrong_number_of_arguments, default_wrong_type, RoocFunction,
};
use crate::parser::il::PreExp;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{
    primitives::{IterableKind, Primitive, PrimitiveKind},
    type_checker::type_checker_context::{TypeCheckerContext, WithType},
};

#[derive(Debug, Serialize, Clone)]
pub struct EdgesOfGraphFn {}
impl RoocFunction for EdgesOfGraphFn {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args[..] {
            [ref of_graph] => {
                let graph = of_graph.as_graph(context, fn_context)?;
                let edges = graph.to_edges();
                Ok(Primitive::Iterable(IterableKind::Edges(edges)))
            }
            _ => Err(default_wrong_number_of_arguments(self)),
        }
    }

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![("of_graph".to_string(), PrimitiveKind::Graph)]
    }

    fn get_return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }

    fn get_function_name(&self) -> String {
        "edges".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        default_type_check(args, &self.get_type_signature(), context, fn_context)
    }
}

#[wasm_bindgen]
#[derive(Debug, Serialize, Clone)]

pub struct NodesOfGraphFn {}

impl RoocFunction for NodesOfGraphFn {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args[..] {
            [ref of_graph] => {
                let graph = of_graph.as_graph(context, fn_context)?;
                let nodes = graph.to_nodes();
                Ok(Primitive::Iterable(IterableKind::Nodes(nodes)))
            }
            _ => Err(default_wrong_number_of_arguments(self)),
        }
    }

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![("of_graph".to_string(), PrimitiveKind::Graph)]
    }

    fn get_return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphNode))
    }

    fn get_function_name(&self) -> String {
        "nodes".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        default_type_check(args, &self.get_type_signature(), context, fn_context)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NeighbourOfNodeFn {}
impl RoocFunction for NeighbourOfNodeFn {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args[..] {
            [ref of_node] => {
                let node = of_node.as_node(context, fn_context)?;
                Ok(Primitive::Iterable(IterableKind::Edges(node.to_edges())))
            }
            _ => Err(default_wrong_number_of_arguments(self)),
        }
    }

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![("of_node".to_string(), PrimitiveKind::GraphNode)]
    }

    fn get_return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }

    fn get_function_name(&self) -> String {
        "neigh_edges".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        default_type_check(args, &self.get_type_signature(), context, fn_context)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NeighboursOfNodeInGraphFn {}
impl RoocFunction for NeighboursOfNodeInGraphFn {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        match args[..] {
            [ref of_node, ref in_graph] => {
                let node = of_node.as_string(context, fn_context)?;
                let graph = in_graph.as_graph(context, fn_context)?;
                let neighbours = graph.into_neighbours_of(&node)?;
                Ok(Primitive::Iterable(IterableKind::Edges(neighbours)))
            }
            _ => Err(default_wrong_number_of_arguments(self)),
        }
    }

    fn get_type_signature(&self) -> Vec<(String, PrimitiveKind)> {
        vec![
            ("of_node_name".to_string(), PrimitiveKind::String),
            ("in_graph".to_string(), PrimitiveKind::Graph),
        ]
    }

    fn get_return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }

    fn get_function_name(&self) -> String {
        "neigh_edges_of".to_string()
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        match args[..] {
            [ref of_node, ref in_graph] => {
                if !matches!(of_node.get_type(context, fn_context), PrimitiveKind::String) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::String,
                        of_node.get_type(context, fn_context),
                        of_node.get_span().clone(),
                    ))
                } else if !matches!(in_graph.get_type(context, fn_context), PrimitiveKind::Graph) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Graph,
                        in_graph.get_type(context, fn_context),
                        in_graph.get_span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Err(default_wrong_type(args, self, context, fn_context)),
        }
    }
}
