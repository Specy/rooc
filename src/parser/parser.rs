use crate::bail_missing_token;
use crate::math_enums::{Comparison, OptimizationType};
use crate::primitives::consts::Constant;
use crate::primitives::parameter::Parameter;
use crate::primitives::primitive::Primitive;
use crate::utils::{CompilationError, InputSpan, ParseError, Spanned};
use pest::iterators::Pair;
use pest::Parser;
use std::fmt::Debug;

use super::pre_exp::PreExp;
use super::rules_parser::{parse_condition_list, parse_consts_declaration, parse_objective};
use super::transformer::{
    transform_parsed_problem, Problem, TransformError, TransformerContext, VariableType,
};

/*
   TODO: add bounds to variables, including wildcards (or add a way to define variable types)
*/

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct PLParser;

#[derive(Debug)]
pub struct IterableSet {
    pub var: VariableType,
    pub iterator: Spanned<Parameter>,
    pub span: InputSpan,
}
impl IterableSet {
    pub fn new(var: VariableType, iterator: Spanned<Parameter>, span: InputSpan) -> Self {
        Self {
            var,
            iterator,
            span,
        }
    }
}

#[derive(Debug)]
pub struct ArrayAccess {
    pub name: String,
    pub accesses: Vec<Parameter>,
}
impl ArrayAccess {
    pub fn new(name: String, accesses: Vec<Parameter>) -> Self {
        Self { name, accesses }
    }
    pub fn to_string(&self) -> String {
        let rest = self
            .accesses
            .iter()
            .map(|a| format!("[{}]", a.to_string()))
            .collect::<Vec<String>>()
            .join("");
        format!("{}{}", self.name, rest)
    }
}

#[derive(Debug)]
pub struct CompoundVariable {
    pub name: String,
    pub indexes: Vec<String>,
}
impl CompoundVariable {
    pub fn new(name: String, indexes: Vec<String>) -> Self {
        Self { name, indexes }
    }
    pub fn to_string(&self) -> String {
        format!("{}_{}", self.name, self.indexes.join("_"))
    }
}

#[derive(Debug)]
pub struct PreObjective {
    pub objective_type: OptimizationType,
    pub rhs: PreExp,
}

impl PreObjective {
    pub fn new(objective_type: OptimizationType, rhs: PreExp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
}

#[derive(Debug)]
pub struct PreCondition {
    pub lhs: PreExp,
    pub condition_type: Comparison,
    pub rhs: PreExp,
    pub iteration: Vec<IterableSet>,
    pub span: InputSpan,
}

impl PreCondition {
    pub fn new(
        lhs: PreExp,
        condition_type: Comparison,
        rhs: PreExp,
        iteration: Vec<IterableSet>,
        span: InputSpan,
    ) -> Self {
        Self {
            lhs,
            condition_type,
            rhs,
            iteration,
            span,
        }
    }
}

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
