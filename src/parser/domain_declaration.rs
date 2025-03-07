#[allow(unused_imports)]
use crate::prelude::*;
use indexmap::IndexMap;
use serde::Serialize;
use std::fmt::Display;

use super::recursive_set_resolver::recursive_set_resolver;
use crate::math::PreVariableType;
use crate::model_transformer::DomainVariable;
use crate::parser::il::CompoundVariable;
use crate::parser::il::IterableSet;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::TransformerContext;
use crate::type_checker::type_checker_context::FunctionContext;
use crate::{
    math::VariableType,
    traits::{escape_latex, ToLatex},
    type_checker::type_checker_context::{TypeCheckable, TypeCheckerContext},
    utils::{InputSpan, Spanned},
};

/// Represents a variable or compound variable
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Variable {
    /// Simple variable name
    Variable(String),
    /// Compound variable with indexes
    CompoundVariable(CompoundVariable),
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Variable(name) => {
                if name.contains("_") {
                    //if it's a variable to be escaped
                    write!(f, "\\{}", name)
                } else {
                    write!(f, "{}", name)
                }
            }
            Variable::CompoundVariable(c) => write!(f, "{}", c),
        }
    }
}

impl ToLatex for Variable {
    fn to_latex(&self) -> String {
        match self {
            Variable::Variable(name) => {
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
            Variable::CompoundVariable(c) => c.to_latex(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IVariableToAssert: &str = r#"
export type SerializedVariableToAssert = {
    type: "Variable",
    value: string,
} | {
    type: "CompoundVariable",
    value: SerializedCompoundVariable,
}
"#;

/// Represents a single domain declaration for variables, defining their type and iteration scope
#[derive(Debug, Clone, Serialize)]
pub struct VariablesDomainDeclaration {
    /// Variables included in this domain
    variables: Vec<Spanned<Variable>>,
    /// Type for all variables in this domain
    as_type: PreVariableType,
    /// Optional iteration scopes to iterate compound variables
    iteration: Vec<IterableSet>,
    /// Source code span for error reporting
    span: InputSpan,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IVariablesDomainDeclaration: &'static str = r#"
export type SerializedVariablesDomainDeclaration = {
    variables: SerializedSpanned<SerializedVariableToAssert>[],
    as_type: VariableType,
    iteration: SerializedIterableSet[],
    span: InputSpan,
}
"#;

impl VariablesDomainDeclaration {
    /// Creates a new domain declaration
    pub fn new(
        variables: Vec<Spanned<Variable>>,
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

    /// Returns reference to variables in this domain
    pub fn variables(&self) -> &Vec<Spanned<Variable>> {
        &self.variables
    }

    /// Returns reference to the type of variables in this domain
    pub fn get_type(&self) -> &PreVariableType {
        &self.as_type
    }

    /// Returns static (non-compound) variables from this domain
    pub fn static_variables(&self) -> Vec<Spanned<String>> {
        self.variables
            .iter()
            .filter_map(|v| match &v.value() {
                Variable::Variable(name) => Some(Spanned::new(name.clone(), v.span().clone())),
                _ => None,
            })
            .collect()
    }

    /// Returns reference to iteration sets
    pub fn iteration(&self) -> &Vec<IterableSet> {
        &self.iteration
    }

    /// Computes the domain values for the current context state
    fn compute_domain_values(
        &self,
        context: &mut TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Vec<(String, Spanned<VariableType>)>, TransformError> {
        self.variables
            .iter()
            .map(|v| {
                match v.value() {
                    Variable::Variable(name) => {
                        let var_type = self.as_type.to_variable_type(context, fn_context)?;
                        Ok((name.clone(), var_type))
                    }
                    Variable::CompoundVariable(c) => {
                        let indexes = &c.compute_indexes(context, fn_context)?;
                        let name = context.flatten_compound_variable(&c.name, indexes)?;
                        let var_type = self.as_type.to_variable_type(context, fn_context)?;
                        Ok((name, var_type))
                    }
                }
                .map(|(name, t)| (name, Spanned::new(t, v.span().clone())))
            })
            .collect::<Result<Vec<(String, Spanned<VariableType>)>, TransformError>>()
            .map_err(|e| e.add_span(&self.span))
    }

    /// Computes the complete domain by evaluating all iterations
    pub fn compute_domain(
        &self,
        context: &mut TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Vec<(String, Spanned<VariableType>)>, TransformError> {
        if self.iteration.is_empty() {
            return self.compute_domain_values(context, fn_context);
        }
        let mut results: Vec<Vec<(String, Spanned<VariableType>)>> = Vec::new();
        recursive_set_resolver(
            &self.iteration,
            context,
            fn_context,
            &mut results,
            0,
            &|context| self.compute_domain_values(context, fn_context),
        )
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
                .map_err(|e| e.add_span(iter.iterator.span()))?;
            context.add_scope();
            let types = iter.variable_types(context, fn_context)?;
            for (name, t) in types {
                context.add_token_type(t, name.span().clone(), Some(name.value().clone()))?;
            }
        }
        for variable in &self.variables {
            match &variable.value() {
                Variable::Variable(name) => {
                    if context.value_of(name).is_some() {
                        return Err(TransformError::Other(format!(
                            "Variable {} already declared as static",
                            name
                        ))
                        .add_span(variable.span()));
                    }
                }
                Variable::CompoundVariable(c) => {
                    context
                        .check_compound_variable(&c.indexes, fn_context)
                        .map_err(|e| e.add_span(variable.span()))?;
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
            if let Variable::CompoundVariable(c) = variable.value() {
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

pub(crate) fn format_domain(domain: &IndexMap<String, DomainVariable>) -> String {
    // First collect all variables into vectors by type and domain
    let mut domain_groups: IndexMap<String, Vec<String>> = IndexMap::new();

    // Group variables by type and domain
    for (name, var) in domain {
        let type_str = var.get_type().to_string();
        domain_groups
            .entry(type_str)
            .or_default()
            .push(name.clone());
    }

    // Format each group
    let mut result = String::new();

    for (type_str, vars) in domain_groups {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&format!("{} as {}", vars.join(", "), type_str));
    }

    result
}
