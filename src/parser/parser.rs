use std::fmt::Debug;

use pest::iterators::Pair;
use pest::Parser;

use crate::bail_missing_token;
use crate::primitives::consts::Constant;
use crate::utils::{CompilationError, ParseError};

use super::pre_parsed_problem::{PreCondition, PreObjective};
use super::rules_parser::other_parser::{parse_condition_list, parse_consts_declaration, parse_objective};
use super::transformer::{Problem, transform_parsed_problem};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct PLParser;

#[derive(Debug, Serialize)]
pub struct PreProblem {
    pub objective: PreObjective,
    pub conditions: Vec<PreCondition>,
    pub constants: Vec<Constant>,
}

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
}

pub fn parse_problem_source(source: &str) -> Result<PreProblem, String> {
    let source = source.trim();
    let problem = PLParser::parse(Rule::problem, source);
    match problem {
        Ok(mut problem) => {
            let problem = problem.next();
            if problem.is_none() {
                return Err("No problem found".to_string());
            }
            let problem = problem.unwrap();
            match parse_problem(problem) {
                Ok(problem) => Ok(problem),
                Err(err) => Err(err.to_string_from_source(source)),
            }
        }
        Err(err) => Err(err.to_string()),
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

