#[allow(unused_imports)]
use crate::prelude::*;
use core::fmt;
use indexmap::IndexMap;
use pest::iterators::Pair;
use pest::Parser;
use serde::Serialize;
use std::fmt::Debug;

#[allow(unused)]
use crate::{bail_missing_token, Primitive};
use crate::math::PreVariableType;
use crate::parser::il::{PreConstraint, PreObjective};
use crate::parser::model_transformer::assert_no_duplicates_in_domain;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::{transform_parsed_problem, Model};
use crate::primitives::{Constant};
#[cfg(target_arch = "wasm32")]
use crate::runtime_builtin::JsFunction;
use crate::runtime_builtin::{make_std, RoocFunction};
use crate::traits::ToLatex;
use crate::type_checker::type_checker_context::{
    FunctionContext, TypeCheckable, TypeCheckerContext, TypedToken,
};
use crate::utils::{CompilationError, InputSpan, ParseError, Spanned};

use super::domain_declaration::VariablesDomainDeclaration;
use super::rules_parser::{
    parse_constraint_list, parse_consts_declaration, parse_domains_declaration, parse_objective,
};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub(crate) struct PLParser;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
/// Represents a pre-transformed optimization model containing the objective,
/// constraints, constants and domain declarations
#[derive(Debug, Serialize, Clone)]
pub struct PreModel {
    /// Original source code if available
    source: Option<String>,
    /// Objective function to optimize
    objective: PreObjective,
    /// List of constraints that must be satisfied
    constraints: Vec<PreConstraint>,
    /// Constant declarations
    constants: Vec<Constant>,
    /// Domain declarations for variables
    domains: Vec<VariablesDomainDeclaration>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IPreProblem: &'static str = r#"
export type SerializedPreModel = {
    objective: SerializedPreObjective,
    constraints: SerializedPreConstraint[],
    constants: SerializedConstant[],
    domains: SerializedVariablesDomainDeclaration[],
}
"#;

impl PreModel {
    pub fn new(
        objective: PreObjective,
        constraint: Vec<PreConstraint>,
        constants: Vec<Constant>,
        domains: Vec<VariablesDomainDeclaration>,
        source: Option<String>,
    ) -> Self {
        Self {
            objective,
            constraints: constraint,
            constants,
            domains,
            source,
        }
    }

    /// Decomposes the model into its constituent parts
    pub fn into_parts(
        self,
    ) -> (
        PreObjective,
        Vec<PreConstraint>,
        Vec<Constant>,
        Vec<VariablesDomainDeclaration>,
        Option<String>,
    ) {
        (
            self.objective,
            self.constraints,
            self.constants,
            self.domains,
            self.source,
        )
    }

    pub fn objective(&self) -> &PreObjective {
        &self.objective
    }
    pub fn constraints(&self) -> &Vec<PreConstraint> {
        &self.constraints
    }
    pub fn constants(&self) -> &Vec<Constant> {
        &self.constants
    }
    pub fn domains(&self) -> &Vec<VariablesDomainDeclaration> {
        &self.domains
    }
    pub fn transform(
        self,
        constants: Vec<Constant>,
        fns: &IndexMap<String, Box<dyn RoocFunction>>,
    ) -> Result<Model, TransformError> {
        transform_parsed_problem(self, constants, fns)
    }
    pub fn source(&self) -> Option<String> {
        self.source.clone()
    }
    fn static_variables_domain(&self) -> Vec<(String, Spanned<PreVariableType>)> {
        self.domains
            .iter()
            .flat_map(|d| {
                d.static_variables().into_iter().map(|v| {
                    let (name, span) = v.into_tuple();
                    (name, Spanned::new(d.get_type().clone(), span))
                })
            })
            .collect::<Vec<_>>()
    }
    pub fn create_type_checker(
        &self,
        constants: &Vec<Constant>,
        fns: &IndexMap<String, Box<dyn RoocFunction>>,
    ) -> Result<(), TransformError> {
        let mut context = TypeCheckerContext::default();
        let domain = self.static_variables_domain();
        let std = make_std();
        let fn_context = FunctionContext::new(fns, &std);
        //TODO add span
        assert_no_duplicates_in_domain(
            &domain
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        Spanned::new(v.to_variable_type_without_context(), v.span().clone()),
                    )
                })
                .collect::<Vec<_>>(),
        )?;
        context.set_static_domain(domain);
        for constant in constants {
            constant.type_check(&mut context, &fn_context)?
        }
        for constant in &self.constants {
            constant.type_check(&mut context, &fn_context)?;
        }
        for domain in &self.domains {
            domain.type_check(&mut context, &fn_context)?;
        }
        self.type_check(&mut context, &fn_context)
    }
    pub fn create_token_type_map(
        &self,
        constants: &Vec<Constant>,
        fns: &IndexMap<String, Box<dyn RoocFunction>>,
    ) -> IndexMap<u32, TypedToken> {
        let mut context = TypeCheckerContext::default();
        let domain = self.static_variables_domain();
        let std = make_std();
        let fn_context = FunctionContext::new(fns, &std);
        context.set_static_domain(domain);
        for constant in constants {
            constant.populate_token_type_map(&mut context, &fn_context);
        }
        for constant in &self.constants {
            constant.populate_token_type_map(&mut context, &fn_context);
        }
        for domain in &self.domains {
            domain.populate_token_type_map(&mut context, &fn_context);
        }
        self.populate_token_type_map(&mut context, &fn_context);
        context.into_token_map()
    }
}

impl TypeCheckable for PreModel {
    fn type_check(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) -> Result<(), TransformError> {
        self.objective.type_check(context, fn_context)?;
        for domain in &self.domains {
            domain.type_check(context, fn_context)?;
        }
        for cond in &self.constraints {
            cond.type_check(context, fn_context)?;
        }
        Ok(())
    }
    fn populate_token_type_map(
        &self,
        context: &mut TypeCheckerContext,
        fn_context: &FunctionContext,
    ) {
        self.objective.populate_token_type_map(context, fn_context);
        for domain in &self.domains {
            domain.populate_token_type_map(context, fn_context);
        }
        for cond in &self.constraints {
            cond.populate_token_type_map(context, fn_context);
        }
    }
}

impl ToLatex for PreModel {
    fn to_latex(&self) -> String {
        let mut s = self.objective.to_latex();
        s.push_str("\\\\\n{s.t.}\\\\\n");
        let constraints = self
            .constraints
            .iter()
            .map(|cond| format!("    \\quad {} \\quad", cond.to_latex()))
            .collect::<Vec<_>>()
            .join("\\\\\n");
        s.push_str(format!("\n\\begin{{align}}\n{}\n\\end{{align}}", constraints).as_str());
        if !self.constants.is_empty() {
            s.push_str("\\\\\n where \\\\\n");
            let constants = self
                .constants
                .iter()
                .map(|constant| format!("     \\quad {}", constant.to_latex()))
                .collect::<Vec<_>>()
                .join("\\\\\n");
            s.push_str(format!("\n\\begin{{align*}}\n{}\n\\end{{align*}}", constants).as_str());
        }
        if !self.domains.is_empty() {
            s.push_str("\\\\\n define \\\\\n");
            let domains = self
                .domains
                .iter()
                .map(|domain| format!("     \\quad {}", domain.to_latex()))
                .collect::<Vec<_>>()
                .join("\\\\\n");
            s.push_str(format!("\n\\begin{{align*}}\n{}\n\\end{{align*}}", domains).as_str());
        }
        s
    }
}

#[cfg(target_arch = "wasm32")]
pub fn js_value_to_fns_map(fns: Vec<JsFunction>) -> IndexMap<String, Box<dyn RoocFunction>> {
    fns.into_iter()
        .map(|f| {
            (
                f.function_name().clone(),
                Box::new(f) as Box<dyn RoocFunction>,
            )
        })
        .collect()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl PreModel {
    pub fn transform_wasm(
        self,
        constants: JsValue,
        fns: Vec<JsFunction>,
    ) -> Result<Model, TransformErrorWrapper> {
        let constants: Vec<(String, Primitive)> = serde_wasm_bindgen::from_value(constants)
            .map_err(|e| TransformErrorWrapper {
                error: TransformError::Other(e.to_string()),
            })?;
        let constants = constants
            .into_iter()
            .map(|v| Constant::from_primitive(&v.0, v.1))
            .collect();
        let fns = js_value_to_fns_map(fns);
        self.transform(constants, &fns)
            .map_err(|e| TransformErrorWrapper { error: e })
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
    pub fn format_wasm(&self) -> String {
        self.to_string()
    }
    pub fn type_check_wasm(
        self,
        constants: JsValue,
        fns: Vec<JsFunction>,
    ) -> Result<(), TransformErrorWrapper> {
        let constants: Vec<(String, Primitive)> = serde_wasm_bindgen::from_value(constants)
            .map_err(|e| TransformErrorWrapper {
                error: TransformError::Other(e.to_string()),
            })?;
        let constants = constants
            .into_iter()
            .map(|v| Constant::from_primitive(&v.0, v.1))
            .collect();
        let fns = js_value_to_fns_map(fns);
        self.create_type_checker(&constants, &fns)
            .map(|_| ())
            .map_err(|e| TransformErrorWrapper { error: e })
    }
    pub fn create_token_type_map_wasm(&self, constants: JsValue, fns: Vec<JsFunction>) -> JsValue {
        let fns = js_value_to_fns_map(fns);
        let constants: Vec<(String, Primitive)> =
            serde_wasm_bindgen::from_value(constants).unwrap_or_default();
        let constants = constants
            .into_iter()
            .map(|v| Constant::from_primitive(&v.0, v.1))
            .collect();
        serde_wasm_bindgen::to_value(&self.create_token_type_map(&constants, &fns)).unwrap()
    }
    pub fn to_latex_wasm(&self) -> String {
        self.to_latex()
    }

    pub fn wasm_get_source(&self) -> Option<String> {
        self.source.clone()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
pub struct TransformErrorWrapper {
    error: TransformError,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl TransformErrorWrapper {
    pub fn get_trace(&self) -> JsValue {
        let a = self
            .error
            .trace()
            .into_iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();
        serde_wasm_bindgen::to_value(&a).unwrap()
    }
    pub fn origin_span(&self) -> Option<InputSpan> {
        self.error.origin_span()
    }
    pub fn base_error(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.error.base_error()).unwrap()
    }
    pub fn stringify_base_error(&self) -> String {
        self.error.base_error().to_string()
    }
    pub fn traced_error(&self) -> String {
        self.error.traced_error()
    }
    pub fn error_from_source(&self, source: &str) -> Result<String, String> {
        self.error.trace_from_source(source)
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.error).unwrap()
    }
}

impl fmt::Display for PreModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.objective.to_string();
        s.push_str("\ns.t.\n");
        for cond in &self.constraints {
            s.push_str(&format!("    {}\n", cond));
        }
        if !self.constants.is_empty() {
            s.push_str("where\n");
            for constant in &self.constants {
                let constant = constant
                    .to_string()
                    .split("\n")
                    .collect::<Vec<_>>()
                    .join("\n    ");
                s.push_str(&format!("    {}\n", constant));
            }
        }
        if !self.domains.is_empty() {
            s.push_str("define\n");
            for domain in &self.domains {
                let domain = domain
                    .to_string()
                    .split("\n")
                    .collect::<Vec<_>>()
                    .join("\n    ");
                s.push_str(&format!("    {}\n", domain));
            }
        }
        f.write_str(&s)
    }
}

pub fn parse_problem_source(source: &str) -> Result<PreModel, CompilationError> {
    let problem = PLParser::parse(Rule::problem, source);
    match problem {
        Ok(mut problem) => {
            let problem = problem.next();
            if problem.is_none() {
                return Err(CompilationError::new(
                    ParseError::MissingToken("Failed to parse, missing problem".to_string()),
                    InputSpan::default(),
                    source.to_string(),
                ));
            }
            let problem = problem.unwrap();
            parse_problem(problem, source)
        }
        Err(err) => {
            let location = &err.location;
            let span = match location {
                pest::error::InputLocation::Pos(pos) => InputSpan {
                    start: *pos as u32,
                    len: 1,
                    start_line: 0,
                    start_column: 0,
                    tempered: false,
                },
                pest::error::InputLocation::Span((start, end)) => InputSpan {
                    start: *start as u32,
                    len: (end - start) as u32,
                    start_line: 0,
                    start_column: 0,
                    tempered: false,
                },
            };
            let kind = ParseError::UnexpectedToken(err.to_string());
            Err(CompilationError::new(kind, span, source.to_string()))
        }
    }
}

fn parse_problem(problem: Pair<Rule>, source: &str) -> Result<PreModel, CompilationError> {
    let pairs = problem.clone().into_inner();
    let objective = pairs.find_first_tagged("objective").map(parse_objective);
    let constraints = pairs
        .find_first_tagged("constraints")
        .map(|v| parse_constraint_list(&v));
    let consts = pairs
        .find_first_tagged("where")
        .map(parse_consts_declaration);
    let domain = pairs
        .find_first_tagged("define")
        .map(parse_domains_declaration);
    match (objective, constraints) {
        (Some(obj), Some(cond)) => Ok(PreModel::new(
            obj?,
            cond?,
            consts.unwrap_or(Ok(Vec::new()))?,
            domain.unwrap_or(Ok(Vec::new()))?,
            Some(source.to_owned()),
        )),
        _ => bail_missing_token!("Objective and constraints are required", problem),
    }
}
