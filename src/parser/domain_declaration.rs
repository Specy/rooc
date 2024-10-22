use std::fmt::Display;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use super::recursive_set_resolver::recursive_set_resolver;
use crate::math::math_enums::PreVariableType;
use crate::parser::il::il_problem::CompoundVariable;
use crate::parser::il::iterable_set::IterableSet;
use crate::parser::model_transformer::transform_error::TransformError;
use crate::parser::model_transformer::transformer_context::TransformerContext;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{
    math::math_enums::VariableType,
    traits::latex::{escape_latex, ToLatex},
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext},
    utils::{InputSpan, Spanned},
};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum VariableToAssert {
    Variable(String),
    CompoundVariable(CompoundVariable),
}

impl Display for VariableToAssert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableToAssert::Variable(name) => {
                if name.contains("_") {
                    //if it's a variable to be escaped
                    write!(f, "\\{}", name)
                } else {
                    write!(f, "{}", name)
                }
            }
            VariableToAssert::CompoundVariable(c) => write!(f, "{}", c),
        }
    }
}

impl ToLatex for VariableToAssert {
    fn to_latex(&self) -> String {
        match self {
            VariableToAssert::Variable(name) => {
                if name.contains("_") {
                    let mut indexes = name.split("_").collect::<Vec<&str>>();
                    //sure to have at least one element
                    let first = indexes.remove(0);
                    let rest = indexes.join("");
                    format!("{}_{{{}}}", escape_latex(first), escape_latex(&rest))
                } else {
                    escape_latex(name)
                }
            }
            VariableToAssert::CompoundVariable(c) => c.to_latex(),
        }
    }
}

#[wasm_bindgen(typescript_custom_section)]
const IVariableToAssert: &str = r#"
export type SerializedVariableToAssert = {
    type: "Variable",
    value: string,
} | {
    type: "CompoundVariable",
    value: SerializedCompoundVariable,
}
"#;

#[derive(Debug, Clone, Serialize)]
pub struct VariablesDomainDeclaration {
    variables: Vec<Spanned<VariableToAssert>>,
    as_type: PreVariableType,
    iteration: Vec<IterableSet>,
    span: InputSpan,
}

#[wasm_bindgen(typescript_custom_section)]
const IVariablesDomainDeclaration: &'static str = r#"
export type SerializedVariablesDomainDeclaration = {
    variables: SerializedSpanned<SerializedVariableToAssert>[],
    as_type: VariableType,
    iteration: SerializedIterableSet[],
    span: InputSpan,
}
"#;

impl VariablesDomainDeclaration {
    pub fn new(
        variables: Vec<Spanned<VariableToAssert>>,
        as_type: PreVariableType,
        iters: Vec<IterableSet>,
        span: InputSpan,
    ) -> Self {
        Self {
            variables,
            as_type,
            iteration: iters,
            span,
        }
    }

    pub fn get_variables(&self) -> &Vec<Spanned<VariableToAssert>> {
        &self.variables
    }
    pub fn get_type(&self) -> &PreVariableType {
        &self.as_type
    }
    pub fn get_static_variables(&self) -> Vec<Spanned<String>> {
        self.variables
            .iter()
            .filter_map(|v| match &v.get_span_value() {
                VariableToAssert::Variable(name) => {
                    Some(Spanned::new(name.clone(), v.get_span().clone()))
                }
                _ => None,
            })
            .collect()
    }
    pub fn get_iters(&self) -> &Vec<IterableSet> {
        &self.iteration
    }

    fn compute_domain_values(
        &self,
        context: &mut TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Vec<(String, Spanned<VariableType>)>, TransformError> {
        self.variables
            .iter()
            .map(|v| {
                match v.get_span_value() {
                    VariableToAssert::Variable(name) => {
                        let var_type = self.as_type.to_variable_type(context, fn_context)?;
                        Ok((name.clone(), var_type))
                    }
                    VariableToAssert::CompoundVariable(c) => {
                        let indexes = &c.compute_indexes(context, fn_context)?;
                        let name = context.flatten_compound_variable(&c.name, indexes)?;
                        let var_type = self.as_type.to_variable_type(context, fn_context)?;
                        Ok((name, var_type))
                    }
                }
                .map(|(name, t)| (name, Spanned::new(t, v.get_span().clone())))
            })
            .collect::<Result<Vec<(String, Spanned<VariableType>)>, TransformError>>()
            .map_err(|e| e.add_span(&self.span))
    }
    pub fn compute_domain(
        &self,
        context: &mut TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Vec<(String, Spanned<VariableType>)>, TransformError> {
        if self.iteration.is_empty() {
            return self.compute_domain_values(context, fn_context);
        }
        let mut results: Vec<Vec<(String, Spanned<VariableType>)>> = Vec::new();
        recursive_set_resolver(&self.iteration, context,fn_context, &mut results, 0, &|context| {
            self.compute_domain_values(context, fn_context)
        })
        .map_err(|e| e.add_span(&self.span))?;
        Ok(results.into_iter().flatten().collect())
    }
}

impl TypeCheckable for VariablesDomainDeclaration {
    fn type_check(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        for iter in &self.iteration {
            iter.iterator
                .type_check(context, fn_context)
                .map_err(|e| e.add_span(iter.iterator.get_span()))?;
            context.add_scope();
            let types = iter.get_variable_types(context, fn_context)?;
            for (name, t) in types {
                context.add_token_type(
                    t,
                    name.get_span().clone(),
                    Some(name.get_span_value().clone()),
                )?;
            }
        }
        for variable in &self.variables {
            match &variable.get_span_value() {
                VariableToAssert::Variable(name) => {
                    if context.get_value(name).is_some() {
                        return Err(TransformError::Other(format!(
                            "Variable {} already declared as static",
                            name
                        ))
                        .add_span(variable.get_span()));
                    }
                }
                VariableToAssert::CompoundVariable(c) => {
                    context
                        .check_compound_variable(&c.indexes, fn_context)
                        .map_err(|e| e.add_span(variable.get_span()))?;
                }
            }
        }
        self.as_type
            .type_check(context, fn_context)
            .map_err(|e| e.add_span(&self.span))?;
        for _ in &self.iteration {
            context.pop_scope()?;
        }
        Ok(())
    }
    fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) {
        for iter in &self.iteration {
            context.add_scope();
            iter.populate_token_type_map(context, fn_context);
        }

        self.as_type.populate_token_type_map(context, fn_context);
        for variable in &self.variables {
            if let VariableToAssert::CompoundVariable(c) = variable.get_span_value() {
                for index in &c.indexes {
                    index.populate_token_type_map(context, fn_context);
                }
            }
        }

        for _ in &self.iteration {
            let _ = context.pop_scope();
        }
    }
}

impl ToLatex for VariablesDomainDeclaration {
    fn to_latex(&self) -> String {
        let mut s = String::new();
        let vars = self
            .variables
            .iter()
            .map(|v| v.to_latex())
            .collect::<Vec<String>>()
            .join(", ");
        s.push_str(format!("{} &\\in {}", vars, self.as_type.to_latex()).as_str());
        if !self.iteration.is_empty() {
            let iters = self
                .iteration
                .iter()
                .map(|iter| format!(" \\forall{{{}}} ", iter.to_latex()))
                .collect::<Vec<String>>()
                .join(",\\");
            s.push_str(format!(" \\quad {}", iters).as_str());
        }
        s
    }
}

impl Display for VariablesDomainDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vars = self
            .variables
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        if !self.iteration.is_empty() {
            let iters = self
                .iteration
                .iter()
                .map(|iter| iter.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "{} as {} for {}", vars, self.as_type, iters)
        } else {
            write!(f, "{} as {}", vars, self.as_type)
        }
    }
}
