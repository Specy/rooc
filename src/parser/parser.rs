use core::fmt;
use std::fmt::Debug;

use pest::iterators::Pair;
use pest::Parser;

use crate::bail_missing_token;
use crate::primitives::consts::Constant;
use crate::utils::{CompilationError, InputSpan, ParseError};

use super::pre_parsed_problem::{PreCondition, PreObjective};
use super::rules_parser::other_parser::{
    parse_condition_list, parse_consts_declaration, parse_objective,
};
use super::transformer::{transform_parsed_problem, Problem, TransformError};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct PLParser;

#[wasm_bindgen]
#[derive(Debug, Serialize)]
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
        transform_parsed_problem(&self)
    }
}

#[wasm_bindgen]
impl PreProblem {
    pub fn transform_wasm(self) -> Result<Problem, JsValue> {
        self.transform()
            .map_err(|e| serde_wasm_bindgen::to_value(&e).unwrap())
    }
    pub fn serialize_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap()
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
