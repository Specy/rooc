use std::fmt::Debug;

use pest::iterators::Pair;
use serde::Serialize;

use crate::{
    bail_wrong_argument_spanned, bail_wrong_number_of_arguments,
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformError, TransformerContext, TypeCheckerContext},
    },
    primitives::{
        iterable::IterableKind,
        primitive::{Primitive, PrimitiveKind},
    },
    utils::{CompilationError, ParseError},
};

use super::function_traits::FunctionCall;

#[derive(Debug, Serialize, Clone)]
pub struct EdgesOfGraphFn {
    of_graph: PreExp,
}

impl FunctionCall for EdgesOfGraphFn {
    fn from_parameters(mut pars: Vec<PreExp>, rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                of_graph: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, rule, "edges", ["Graph"]),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let graph = self.of_graph.as_graph(context)?;
        let edges = graph.to_edges();
        Ok(Primitive::Iterable(IterableKind::Edges(edges)))
    }
    fn to_string(&self) -> String {
        format!("edges({})", self.of_graph)
    }
    fn get_return_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }
    fn type_check(&self, context: &TypeCheckerContext) -> Result<(), TransformError> {
        if !matches!(self.of_graph.get_type(context), PrimitiveKind::Graph) {
            Err(TransformError::from_wrong_type(
                PrimitiveKind::Graph,
                self.of_graph.get_type(context),
                self.of_graph.get_span().clone(),
            ))
        } else {
            Ok(())
        }
    }
    fn get_parameters_types(&self) -> Vec<PrimitiveKind> {
        vec![PrimitiveKind::Graph]
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NodesOfGraphFn {
    of_graph: PreExp,
}
impl FunctionCall for NodesOfGraphFn {
    fn from_parameters(mut pars: Vec<PreExp>, rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                of_graph: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, rule, "nodes", ["Graph"]),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let graph = self.of_graph.as_graph(context)?;
        let nodes = graph.to_nodes();
        Ok(Primitive::Iterable(IterableKind::Nodes(nodes)))
    }
    fn to_string(&self) -> String {
        format!("nodes({})", self.of_graph)
    }
    fn get_return_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphNode))
    }
    fn type_check(&self, context: &TypeCheckerContext) -> Result<(), TransformError> {
        if !matches!(self.of_graph.get_type(context), PrimitiveKind::Graph) {
            Err(TransformError::from_wrong_type(
                PrimitiveKind::Graph,
                self.of_graph.get_type(context),
                self.of_graph.get_span().clone(),
            ))
        } else {
            Ok(())
        }
    }
    fn get_parameters_types(&self) -> Vec<PrimitiveKind> {
        vec![PrimitiveKind::Graph]
    }

}

#[derive(Debug, Serialize, Clone)]
pub struct NeighbourOfNodeFn {
    of_node: PreExp,
}

impl FunctionCall for NeighbourOfNodeFn {
    fn from_parameters(mut pars: Vec<PreExp>, rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        match pars.len() {
            1 => Ok(Self {
                of_node: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, rule, "neighs_edges", ["Node"]),
        }
    }

    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let node = self.of_node.as_node(context)?;
        Ok(Primitive::Iterable(IterableKind::Edges(node.to_edges())))
    }

    fn to_string(&self) -> String {
        format!("neighs_edges({})", self.of_node)
    }
    fn get_parameters_types(&self) -> Vec<PrimitiveKind> {
        vec![PrimitiveKind::GraphNode]
    }
    fn get_return_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }
    fn type_check(&self, context: &TypeCheckerContext) -> Result<(), TransformError> {
        if !matches!(self.of_node.get_type(context), PrimitiveKind::GraphNode) {
            Err(TransformError::from_wrong_type(
                PrimitiveKind::GraphNode,
                self.of_node.get_type(context),
                self.of_node.get_span().clone(),
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NeighboursOfNodeInGraphFn {
    of_node: PreExp,
    in_graph: PreExp,
}
impl FunctionCall for NeighboursOfNodeInGraphFn {
    fn from_parameters(mut pars: Vec<PreExp>, rule: &Pair<Rule>) -> Result<Self, CompilationError> {
        match pars.len() {
            2 => Ok(Self {
                of_node: pars.remove(0),
                in_graph: pars.remove(0),
            }),
            n => bail_wrong_number_of_arguments!(n, rule, "neigh_edges_of", ["Node", "Graph"]),
        }
    }
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        let node = self.of_node.as_string(context)?;
        let graph = self.in_graph.as_graph(context)?;
        let neighbours = graph.into_neighbours_of(&node)?;
        Ok(Primitive::Iterable(IterableKind::Edges(neighbours)))
    }
    fn to_string(&self) -> String {
        format!("neigh_edges_of({},{})", self.of_node, self.in_graph)
    }
    fn get_parameters_types(&self) -> Vec<PrimitiveKind> {
        vec![PrimitiveKind::GraphNode, PrimitiveKind::Graph]
    }
    fn get_return_type(&self, _: &TypeCheckerContext) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::GraphEdge))
    }
    fn type_check(&self, context: &TypeCheckerContext) -> Result<(), TransformError> {
        if !matches!(self.of_node.get_type(context), PrimitiveKind::GraphNode) {
            Err(TransformError::from_wrong_type(
                PrimitiveKind::GraphNode,
                self.of_node.get_type(context),
                self.of_node.get_span().clone(),
            ))
        } else if !matches!(self.in_graph.get_type(context), PrimitiveKind::Graph) {
            Err(TransformError::from_wrong_type(
                PrimitiveKind::Graph,
                self.in_graph.get_type(context),
                self.in_graph.get_span().clone(),
            ))
        } else {
            Ok(())
        }
    }
}
