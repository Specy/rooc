use std::fmt::Debug;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::Serialize;

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
pub(crate) struct EdgesOfGraphFn {
    pub shorthand_name: bool,
}
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
            _ => Err(default_wrong_number_of_arguments(self, args, fn_context)),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        vec![("of_graph".to_string(), PrimitiveKind::Graph)]
    }

    fn return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }

    fn function_name(&self) -> String {
        if self.shorthand_name {
            "E".to_string()
        } else {
            "edges".to_string()
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Serialize, Clone)]
pub struct NodesOfGraphFn {
    pub shorthand_name: bool,
}

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
            _ => Err(default_wrong_number_of_arguments(self, args, fn_context)),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        vec![("of_graph".to_string(), PrimitiveKind::Graph)]
    }

    fn return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphNode))
    }

    fn function_name(&self) -> String {
        if self.shorthand_name {
            "V".to_string()
        } else {
            "nodes".to_string()
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NeighbourOfNodeFn {
    pub shorthand_name: bool,
}
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
            _ => Err(default_wrong_number_of_arguments(self, args, fn_context)),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        vec![("of_node".to_string(), PrimitiveKind::GraphNode)]
    }

    fn return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }

    fn function_name(&self) -> String {
        if self.shorthand_name {
            "N".to_string()
        } else {
            "neigh_edges".to_string()
        }
    }

    fn type_check(
        &self,
        args: &[PreExp],
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        default_type_check(
            args,
            &self.type_signature(args, context, fn_context),
            context,
            fn_context,
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NeighboursOfNodeInGraphFn {
    pub shorthand_name: bool,
}
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
            _ => Err(default_wrong_number_of_arguments(self, args, fn_context)),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        vec![
            ("of_node_name".to_string(), PrimitiveKind::String),
            ("in_graph".to_string(), PrimitiveKind::Graph),
        ]
    }

    fn return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }

    fn function_name(&self) -> String {
        if self.shorthand_name {
            "N_of".to_string()
        } else {
            "neigh_edges_of".to_string()
        }
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
                        of_node.span().clone(),
                    ))
                } else if !matches!(in_graph.get_type(context, fn_context), PrimitiveKind::Graph) {
                    Err(TransformError::from_wrong_type(
                        PrimitiveKind::Graph,
                        in_graph.get_type(context, fn_context),
                        in_graph.span().clone(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Err(default_wrong_type(args, self, context, fn_context)),
        }
    }
}
