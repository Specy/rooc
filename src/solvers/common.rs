use crate::math::math_enums::{OptimizationType, VariableType};
use crate::parser::model_transformer::transformer_context::DomainVariable;
use copper::views::{Times, ViewExt};
use copper::{VarId, VarIdBinary};
use num_traits::ToPrimitive;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub enum SolverError {
    InvalidDomain {
        expected: Vec<VariableType>,
        got: Vec<(String, DomainVariable)>,
    },
    TooLarge {
        name: String,
        value: f64,
    },
    DidNotSolve,
    UnimplementedOptimizationType {
        expected: Vec<OptimizationType>,
        got: OptimizationType,
    }
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverError::InvalidDomain { expected, got } => {
                let vars = got
                    .iter()
                    .map(|(name, domain)| format!("    {}: {}", name, domain.get_type()))
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(
                    f,
                    "Invalid domain, the following variables are not {}: \n{}",
                    expected
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(" or "),
                    vars
                )
            }
            SolverError::TooLarge { name, value } => {
                write!(f, "The value of variable {} is too large: {}", name, value)
            }
            SolverError::DidNotSolve => {
                write!(f, "The problem was able to be solved")
            }
            SolverError::UnimplementedOptimizationType { expected, got } => {
                write!(
                    f,
                    "Expected optimization type to be one of {:?} but got {:?}",
                    expected, got
                )
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Assignment<T: Clone + Serialize + Copy> {
    pub name: String,
    pub value: T,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegerBinaryLpSolution<T: Clone + Serialize + Copy> {
    assignment: Vec<Assignment<T>>,
    value: f64,
}

impl<T: Clone + Serialize + Copy> IntegerBinaryLpSolution<T> {
    pub fn new(assignment: Vec<Assignment<T>>, value: f64) -> Self {
        Self { assignment, value }
    }

    pub fn get_assignment(&self) -> &Vec<Assignment<T>> {
        &self.assignment
    }
    pub fn get_assignment_values(&self) -> Vec<T> {
        self.assignment.iter().map(|a| a.value).collect()
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub fn find_invalid_variables<F>(
    domain: &HashMap<String, DomainVariable>,
    validator: F,
) -> Vec<(String, DomainVariable)>
where
    F: Fn(&VariableType) -> bool,
{
    domain
        .iter()
        .filter_map(|(name, var)| {
            let var_type = var.get_type();
            if !validator(var_type) {
                Some((name.clone(), var.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn process_variables<'a, F>(
    coefficients: impl Iterator<Item = &'a f64>,
    variables: impl Iterator<Item = &'a VarId>,
    filter_fn: F,
) -> Option<Vec<Times<VarId>>>
where
    F: Fn(usize) -> bool,
{
    coefficients
        .enumerate()
        .filter(|(i, _c)| filter_fn(*i))
        .zip(variables)
        .map(|((i, c), v)| c.to_i32().map(|c| v.times(c)))
        .collect::<Option<Vec<_>>>()
}

pub fn process_variables_binary<'a, F>(
    coefficients: impl Iterator<Item = &'a f64>,
    variables: impl Iterator<Item = &'a VarIdBinary>,
    filter_fn: F,
) -> Option<Vec<Times<VarIdBinary>>>
where
    F: Fn(usize) -> bool,
{
    coefficients
        .enumerate()
        .filter(|(i, _c)| filter_fn(*i))
        .zip(variables)
        .map(|((i, c), v)| c.to_i32().map(|c| v.times(c)))
        .collect::<Option<Vec<_>>>()
}
