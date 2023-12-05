use std::fmt::Debug;

use crate::{
    bail_wrong_argument_spanned, match_or_bail_spanned,
    parser::{transformer::{TransformError, TransformerContext}, pre_parsed_problem::{CompoundVariable, ArrayAccess}},
    utils::{InputSpan, Spanned},
    wrong_argument,
};

use super::{
    functions::function_traits::FunctionCall,
    graph::{Graph, GraphEdge, GraphNode},
    iterable::IterableKind,
    primitive::Primitive,
};

#[derive(Debug)]
pub enum Parameter {
    Primitive(Spanned<Primitive>),
    Variable(Spanned<String>),
    CompoundVariable(Spanned<CompoundVariable>),
    ArrayAccess(Spanned<ArrayAccess>),
    FunctionCall(Spanned<Box<dyn FunctionCall>>),
}

impl Parameter {
    pub fn as_span(&self) -> &InputSpan {
        match self {
            Parameter::Primitive(p) => p.get_span(),
            Parameter::Variable(s) => s.get_span(),
            Parameter::CompoundVariable(c) => c.get_span(),
            Parameter::ArrayAccess(a) => a.get_span(),
            Parameter::FunctionCall(f) => f.get_span(),
        }
    }

    pub fn as_primitive(&self, context: &TransformerContext) -> Result<Primitive, TransformError> {
        match self {
            Parameter::Primitive(p) => Ok(p.get_span_value().clone()),
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
                let value = context.get_array_value(a)?;
                Ok(value.to_owned())
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
    pub fn as_boolean(&self, context: &TransformerContext) -> Result<bool, TransformError> {
        let value = self
            .as_primitive(context)
            .map_err(|e| e.to_spanned_error(self.as_span()))?;
        match_or_bail_spanned!("Boolean", Primitive::Boolean(b) => Ok(b) ; (value, self))
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
            Parameter::Primitive(p) => p.to_string(),
            Parameter::Variable(s) => s.to_string(),
            Parameter::CompoundVariable(c) => c.to_string(),
            Parameter::ArrayAccess(a) => a.to_string(),
            Parameter::FunctionCall(f) => f.to_string(),
        }
    }
}
