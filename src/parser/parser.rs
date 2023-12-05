use crate::bail_missing_token;
use crate::primitives::consts::Constant;
use crate::utils::{CompilationError, ParseError};
use pest::iterators::Pair;
use pest::Parser;
use std::fmt::Debug;

use super::pre_parsed_problem::{PreCondition, PreObjective};
use super::rules_parser::{parse_condition_list, parse_consts_declaration, parse_objective};
use super::transformer::{transform_parsed_problem, Problem};

/*
   TODO: add bounds to variables, including wildcards (or add a way to define variable types)
*/

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct PLParser;

#[derive(Debug)]
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

pub fn parse_problem_source(source: &String) -> Result<PreProblem, String> {
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

pub struct RoocParser {
    source: String,
}
impl RoocParser {
    pub fn new(source: String) -> Self {
        Self { source }
    }
    pub fn parse(&self) -> Result<PreProblem, String> {
        parse_problem_source(&self.source)
    }
    pub fn parse_and_transform(&self) -> Result<Problem, String> {
        let parsed = self.parse()?;
        let transformed = transform_parsed_problem(&parsed);
        match transformed {
            Ok(transformed) => Ok(transformed),
            Err(e) => Err(e
                .get_trace_from_source(&self.source)
                .unwrap_or(e.get_traced_error())),
        }
    }
}
