use crate::bail_missing_token;
use crate::math_enums::{Comparison, OptimizationType};
use crate::primitives::consts::Constant;
use crate::primitives::functions::{Parameter, ToNum};
use crate::primitives::primitive::Primitive;
use crate::utils::{CompilationError, InputSpan, ParseError, Spanned};
use pest::iterators::Pair;
use pest::Parser;
use std::fmt::Debug;

use super::pre_exp::PreExp;
use super::rules_parser::{parse_condition_list, parse_consts_declaration, parse_objective};
use super::transformer::{NamedSet, TransformError, TransformerContext, VariableType};

/*
   TODO: add bounds to variables, including wildcards (or add a way to define variable types)
*/

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
struct PLParser;

#[derive(Debug)]
pub enum PreIterOfArray {
    Array(String),
    ArrayAccess(ArrayAccess),
}
#[derive(Debug)]
pub struct PreSet {
    pub var: VariableType,
    pub iterator: Spanned<PreIterator>,
    pub span: InputSpan,
}
impl PreSet {
    pub fn new(var: VariableType, iterator: Spanned<PreIterator>, span: InputSpan) -> Self {
        Self {
            var,
            iterator,
            span,
        }
    }
}

#[derive(Debug)]
pub enum PreNode {
    Name(String),
    Variable(String),
}

#[derive(Debug)]
pub enum PreIterator {
    Range {
        from: Box<dyn ToNum>,
        to: Box<dyn ToNum>,
        to_inclusive: bool,
    },
    Parameter(Parameter),
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

pub fn recursive_set_resolver<T>(
    sets: &Vec<NamedSet>,
    context: &mut TransformerContext,
    results: &mut Vec<T>,
    current_level: usize,
    on_leaf: &dyn Fn(&mut TransformerContext) -> Result<T, TransformError>,
) -> Result<(), TransformError> {
    let range = sets.get(current_level).unwrap();
    context.add_scope();
    match &range.var {
        VariableType::Single(n) => {
            context
                .declare_variable(n, Primitive::Undefined, true)
                .map_err(|e| e.to_spanned_error(&range.span))?;
        }
        VariableType::Tuple(t) => {
            for name in t.iter() {
                context
                    .declare_variable(name, Primitive::Undefined, true)
                    .map_err(|e| e.to_spanned_error(&range.span))?;
            }
        }
    }
    for value in range.set.iter() {
        //TODO decide if doing spreadable or not
        if let Primitive::Tuple(t) = value {
            if range.vars.len() > t.len() {
                let error = format!(
                    "Cannot destructure tuple of size {} into {} variables",
                    t.len(),
                    range.vars.len()
                );
                return Err(TransformError::WrongArgument(error).to_spanned_error(&range.span));
            }
        }
        for (i, var) in range.vars.iter().enumerate() {
            match value {
                Primitive::Tuple(t) => {
                    if let Some(value) = t.get(i) {
                        context
                            .update_variable(var, value.clone())
                            .map_err(|e| e.to_spanned_error(var.get_span()))?;
                    } else {
                        let error = format!(
                            "Cannot destructure tuple of size {} into {} variables",
                            t.len(),
                            range.vars.len()
                        );
                        return Err(
                            TransformError::WrongArgument(error).to_spanned_error(&range.span)
                        );
                    }
                }
                _ => {
                    context
                        .update_variable(var, value.clone())
                        .map_err(|e| e.to_spanned_error(var.get_span()))?;
                }
            }
        }
        if current_level + 1 >= sets.len() {
            let value = on_leaf(context)?;
            results.push(value);
        } else {
            recursive_set_resolver(sets, context, results, current_level + 1, on_leaf)
                .map_err(|e| e.to_spanned_error(&range.span))?;
        }
    }
    context.pop_scope()?;
    Ok(())
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
    pub iteration: Vec<PreSet>,
    pub span: InputSpan,
}

impl PreCondition {
    pub fn new(
        lhs: PreExp,
        condition_type: Comparison,
        rhs: PreExp,
        iteration: Vec<PreSet>,
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
pub enum PreLen {
    LenOfArray(String),
    Number(i32),
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
                Err(err) => Err(format!("{}", err.to_string())),
            }
        }
        Err(err) => Err(format!("{:#?}", err)),
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
