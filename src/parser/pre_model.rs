use core::fmt;
use indexmap::IndexMap;
use pest::iterators::Pair;
use pest::Parser;
use serde::Serialize;
use std::fmt::Debug;
use wasm_bindgen::prelude::*;

use crate::bail_missing_token;
use crate::math::PreVariableType;
use crate::parser::il::{PreConstraint, PreObjective};
use crate::parser::model_transformer::assert_no_duplicates_in_domain;
use crate::parser::model_transformer::TransformError;
use crate::parser::model_transformer::{transform_parsed_problem, Model};
use crate::primitives::Constant;
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
struct PLParser;

#[wasm_bindgen]
#[derive(Debug, Serialize, Clone)]
pub struct PreModel {
    source: Option<String>,
    objective: PreObjective,
    constraints: Vec<PreConstraint>,
    constants: Vec<Constant>,
    domains: Vec<VariablesDomainDeclaration>,
}

#[wasm_bindgen(typescript_custom_section)]
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
    pub fn get_objective(&self) -> &PreObjective {
        &self.objective
    }
    pub fn get_constraints(&self) -> &Vec<PreConstraint> {
        &self.constraints
    }
    pub fn get_constants(&self) -> &Vec<Constant> {
        &self.constants
    }
    pub fn get_domains(&self) -> &Vec<VariablesDomainDeclaration> {
        &self.domains
    }
    pub fn transform(self) -> Result<Model, TransformError> {
        transform_parsed_problem(self)
    }
    pub fn get_source(&self) -> Option<String> {
        self.source.clone()
    }
    fn get_static_domain(&self) -> Vec<(String, Spanned<PreVariableType>)> {
        self.domains
            .iter()
            .flat_map(|d| {
                d.get_static_variables().into_iter().map(|v| {
                    let (name, span) = v.into_tuple();
                    (name, Spanned::new(d.get_type().clone(), span))
                })
            })
            .collect::<Vec<_>>()
    }
    pub fn create_type_checker(&self) -> Result<(), TransformError> {
        let mut context = TypeCheckerContext::default();
        let domain = self.get_static_domain();
        let fn_context = FunctionContext::default();
        //TODO add span
        assert_no_duplicates_in_domain(
            &domain
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        Spanned::new(v.to_variable_type_without_context(), v.get_span().clone()),
                    )
                })
                .collect::<Vec<_>>(),
        )?;
        context.set_static_domain(domain);
        for constants in &self.constants {
            constants.type_check(&mut context, &fn_context)?;
        }
        for domain in &self.domains {
            domain.type_check(&mut context, &fn_context)?;
        }
        self.type_check(&mut context, &fn_context)
    }
    pub fn create_token_type_map(&self) -> IndexMap<u32, TypedToken> {
        let mut context = TypeCheckerContext::default();
        let domain = self.get_static_domain();
        let fn_context = FunctionContext::default();
        context.set_static_domain(domain);
        for constants in &self.constants {
            constants.populate_token_type_map(&mut context, &fn_context);
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

#[wasm_bindgen]
impl PreModel {
    pub fn transform_wasm(self) -> Result<Model, TransformErrorWrapper> {
        self.transform()
            .map_err(|e| TransformErrorWrapper { error: e })
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
    }
    pub fn format_wasm(&self) -> String {
        self.to_string()
    }
    pub fn type_check_wasm(self) -> Result<(), TransformErrorWrapper> {
        self.create_type_checker()
            .map(|_| ())
            .map_err(|e| TransformErrorWrapper { error: e })
    }
    pub fn create_token_type_map_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.create_token_type_map()).unwrap()
    }
    pub fn to_latex_wasm(&self) -> String {
        self.to_latex()
    }

    pub fn wasm_get_source(&self) -> Option<String> {
        self.source.clone()
    }
}

#[wasm_bindgen]
pub struct TransformErrorWrapper {
    error: TransformError,
}

#[wasm_bindgen]
impl TransformErrorWrapper {
    pub fn get_trace(&self) -> JsValue {
        let a = self
            .error
            .get_trace()
            .into_iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();
        serde_wasm_bindgen::to_value(&a).unwrap()
    }
    pub fn get_origin_span(&self) -> Option<InputSpan> {
        self.error.get_origin_span()
    }
    pub fn get_base_error(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.error.get_base_error()).unwrap()
    }
    pub fn stringify_base_error(&self) -> String {
        self.error.get_base_error().to_string()
    }
    pub fn get_traced_error(&self) -> String {
        self.error.get_traced_error()
    }
    pub fn get_error_from_source(&self, source: &str) -> Result<String, String> {
        self.error.get_trace_from_source(source)
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

pub fn parse_problem_source(source: &String) -> Result<PreModel, CompilationError> {
    let problem = PLParser::parse(Rule::problem, source);
    match problem {
        Ok(mut problem) => {
            let problem = problem.next();
            if problem.is_none() {
                return Err(CompilationError::new(
                    ParseError::MissingToken("Failed to parse, missing problem".to_string()),
                    InputSpan::default(),
                    source.clone(),
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
