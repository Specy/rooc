use core::fmt;
use std::collections::HashMap;
use std::fmt::Debug;

use pest::iterators::Pair;
use pest::Parser;

use crate::bail_missing_token;
use crate::primitives::consts::Constant;
use crate::type_checker::type_checker_context::{TypeCheckerContext, TypeCheckable, TypedToken};
use crate::utils::{CompilationError, InputSpan, ParseError};

use super::pre_parsed_problem::{PreCondition, PreObjective};
use super::rules_parser::other_parser::{
    parse_condition_list, parse_consts_declaration, parse_objective,
};
use super::transformer::{
    transform_parsed_problem, Problem, TransformError, TransformerContext,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct PLParser;

#[wasm_bindgen]
#[derive(Debug, Serialize, Clone)]
pub struct PreProblem {
    objective: PreObjective,
    conditions: Vec<PreCondition>,
    constants: Vec<Constant>,
}
#[wasm_bindgen(typescript_custom_section)]
const IPreProblem: &'static str = r#"
export type SerializedPreProblem = {
    objective: SerializedPreObjective,
    conditions: SerializedPreCondition[],
    constants: SerializedConstant[]
}
"#;

impl PreProblem {
    pub fn new(
        objective: PreObjective,
        conditions: Vec<PreCondition>,
        constants: Vec<Constant>,
    ) -> Self {
        Self {
            objective,
            conditions,
            constants,
        }
    }
    pub fn get_objective(&self) -> &PreObjective {
        &self.objective
    }
    pub fn get_conditions(&self) -> &Vec<PreCondition> {
        &self.conditions
    }
    pub fn get_constants(&self) -> &Vec<Constant> {
        &self.constants
    }
    pub fn transform(self) -> Result<Problem, TransformError> {
        transform_parsed_problem(self)
    }
    pub fn create_type_checker(&self) -> Result<(), TransformError> {
        let mut context = TypeCheckerContext::new_from_constants(&self.constants);
        self.type_check(&mut context)
    }
    pub fn create_token_type_map(&self) -> HashMap<usize, TypedToken> {
        let mut context = TypeCheckerContext::new_from_constants(&self.constants);
        self.populate_token_type_map(&mut context);
        context.into_token_map()
    }
}

impl TypeCheckable for PreProblem {
    fn type_check(&self, context: &mut TypeCheckerContext) -> Result<(), TransformError> {
        self.objective.type_check(context)?;
        for cond in &self.conditions {
            cond.type_check(context)?;
        }
        Ok(())
    }
    fn populate_token_type_map(&self, context: &mut TypeCheckerContext) {
        self.objective.populate_token_type_map(context);
        for cond in &self.conditions {
            cond.populate_token_type_map(context);
        }
    }
}

#[wasm_bindgen]
impl PreProblem {
    pub fn transform_wasm(self) -> Result<Problem, TransformErrorWrapper> {
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
    pub fn get_traced_error(&self) -> String {
        self.error.get_traced_error()
    }
    pub fn get_error_from_source(&self, source: &str) -> Result<String, String> {
        self.error
            .get_trace_from_source(source)
            .map_err(|_| format!("Error not found in source"))
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.error).unwrap()
    }
}



impl fmt::Display for PreProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.objective.to_string();
        s.push_str("\ns.t.\n");
        for cond in &self.conditions {
            s.push_str(&format!("    {}\n", cond.to_string()));
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
        f.write_str(&s)
    }
}

pub fn parse_problem_source(source: &str) -> Result<PreProblem, CompilationError> {
    let source = source.trim();
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
            parse_problem(problem)
        }
        Err(err) => {
            let location = &err.location;
            let span = match location {
                pest::error::InputLocation::Pos(pos) => InputSpan {
                    start: *pos,
                    len: 1,
                    start_line: 0,
                    start_column: 0,
                    tempered: false,
                },
                pest::error::InputLocation::Span((start, end)) => InputSpan {
                    start: *start,
                    len: end - start,
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

fn parse_problem(problem: Pair<Rule>) -> Result<PreProblem, CompilationError> {
    let pairs = problem.clone().into_inner();
    let objective = pairs.find_first_tagged("objective").map(parse_objective);
    let conditions = pairs
        .find_first_tagged("conditions")
        .map(|v| parse_condition_list(&v));
    let consts = pairs
        .find_first_tagged("where")
        .map(parse_consts_declaration);
    match (objective, conditions) {
        (Some(obj), Some(cond)) => Ok(PreProblem::new(
            obj?,
            cond?,
            consts.unwrap_or(Ok(Vec::new()))?,
        )),
        _ => bail_missing_token!("Objective and conditions are required", problem),
    }
}
