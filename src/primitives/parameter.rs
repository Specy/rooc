use std::fmt::Debug;

use crate::{
    bail_wrong_argument_spanned, match_or_bail_spanned,
    parser::{
        parser::{ArrayAccess, CompoundVariable},
        transformer::{TransformError, TransformerContext},
    },
    utils::{InputSpan, Spanned},
    wrong_argument,
};

use super::{
    functions::function_traits::FunctionCall, graph::{Graph, GraphNode, GraphEdge}, iterable::IterableKind,
    primitive::Primitive,
};

#[derive(Debug)]
pub enum Parameter {
    Number(Spanned<f64>),
    String(Spanned<String>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<ArrayAccess>),
    FunctionCall(Spanned<Box<dyn FunctionCall>>),
}

impl Parameter {
    pub fn as_span(&self) -> &InputSpan {
        match self {
            Parameter::Number(n) => n.get_span(),
            Parameter::String(s) => s.get_span(),
            Parameter::Variable(s) => s.get_span(),
            Parameter::CompoundVariable(c) => c.get_span(),
            Parameter::ArrayAccess(a) => a.get_span(),
            Parameter::FunctionCall(f) => f.get_span(),
        }
    }

    pub fn as_primitive(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self {
            Parameter::Number(n) => Ok(Primitive::Number(**n)),
            Parameter::String(s) => Ok(Primitive::String(s.get_span_value().clone())),
            Parameter::Variable(s) => match context.get_value(s) {
                Some(value) => Ok(value.clone()),
                None => Err(TransformError::MissingVariable(s.get_span_value().clone())),
            },
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;
                match context.get_value(&name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(TransformError::MissingVariable(name)),
                }
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                Ok(value)
            }
            Parameter::ArrayAccess(a) => {
                let value = context.get_array_access_value(a)?;
                Ok(Primitive::Number(value))
            }
        }
    }
    //TODO make this a macro
    pub fn as_number(&self, context: &TransformerContext) -> Result<f64, TransformError> {
        let value = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        match_or_bail_spanned!("Number", Primitive::Number(n) => Ok(n) ; (value, self))
    }
    pub fn as_string(&self, context: &TransformerContext) -> Result<String, TransformError> {
        let value = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        match_or_bail_spanned!("String", Primitive::String(s) => Ok(s.to_owned()) ; (value, self))
    }
    pub fn as_integer(&self, context: &TransformerContext) -> Result<i64, TransformError> {
        let n = self
            .as_number(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        if n.fract() != 0.0 {
            bail_wrong_argument_spanned!("Integer", self)
        } else {
            Ok(n as i64)
        }
    }
    pub fn as_usize(&self, context: &TransformerContext) -> Result<usize, TransformError> {
        let n = self
            .as_number(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        if n.fract() != 0.0 {
            bail_wrong_argument_spanned!("Integer", self)
        } else if n < 0.0 {
            bail_wrong_argument_spanned!("Positive integer", self)
        } else {
            Ok(n as usize)
        }
    }
    pub fn as_graph(&self, context: &TransformerContext) -> Result<Graph, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_graph().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn as_node(&self, context: &TransformerContext) -> Result<GraphNode, TransformError> {
        let node = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        match_or_bail_spanned!("GraphNode", Primitive::GraphNode(n) => Ok(n.to_owned()) ; (node, self))
    }
    pub fn as_edge(&self, context: &TransformerContext) -> Result<GraphEdge, TransformError> {
        let edge = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        match_or_bail_spanned!("GraphEdge", Primitive::GraphEdge(e) => Ok(e.to_owned()) ; (edge, self))
    }
    pub fn as_number_array(
        &self,
        context: &TransformerContext,
    ) -> Result<Vec<f64>, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_number_array().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn as_number_matrix(
        &self,
        context: &TransformerContext,
    ) -> Result<Vec<Vec<f64>>, TransformError> {
        self.as_primitive(context)
            .map(|p| {
                p.as_number_matrix()
                    .map(|v| v.iter().map(|v| (*v).to_owned()).collect::<Vec<_>>())
            })
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn as_iterator(
        &self,
        context: &TransformerContext,
    ) -> Result<IterableKind, TransformError> {
        self.as_primitive(context)
            .map(|p| p.as_iterator().map(|v| v.to_owned()))
            .map_err(|e| e.to_spanned_error(self.as_span()))?
    }
    pub fn to_string(&self) -> String {
        match self {
            Parameter::Number(n) => n.to_string(),
            Parameter::String(s) => s.to_string(),
            Parameter::Variable(s) => s.to_string(),
            Parameter::CompoundVariable(c) => c.to_string(),
            Parameter::ArrayAccess(a) => a.to_string(),
            Parameter::FunctionCall(f) => f.to_string(),
        }
    }
}
