use crate::math::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::DomainVariable;
use copper::views::{Times, ViewExt};
use copper::{VarId, VarIdBinary};
use indexmap::IndexMap;
use num_traits::ToPrimitive;
use serde::Serialize;

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
    Unbounded,
    Infisible,
    Other(String),
    LimitReached,
    UnimplementedOptimizationType {
        expected: Vec<OptimizationType>,
        got: OptimizationType,
    },
    UnavailableComparison {
        got: Comparison,
        expected: Vec<Comparison>,
    },
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
            SolverError::Unbounded => {
                write!(f, "The problem is unbounded")
            }
            SolverError::Other(s) => {
                write!(f, "{}", s)
            }
            SolverError::LimitReached => {
                write!(f, "The iteration limit was reached")
            }
            SolverError::UnavailableComparison { got, expected } => {
                write!(
                    f,
                    "The comparison \"{}\" is not available in this solver, expected one of {}",
                    got,
                    expected
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SolverError::TooLarge { name, value } => {
                write!(f, "The value of variable {} is too large: {}", name, value)
            }
            SolverError::DidNotSolve => {
                write!(f, "The problem was able to be solved")
            }
            SolverError::Infisible => {
                write!(f, "The problem is infeasible")
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
pub struct LpSolution<T: Clone + Serialize + Copy> {
    assignment: Vec<Assignment<T>>,
    value: f64,
}

impl<T: Clone + Serialize + Copy> LpSolution<T> {
    pub fn new(assignment: Vec<Assignment<T>>, value: f64) -> Self {
        Self { assignment, value }
    }

    pub fn assignment(&self) -> &Vec<Assignment<T>> {
        &self.assignment
    }
    pub fn assignment_values(&self) -> Vec<T> {
        self.assignment.iter().map(|a| a.value).collect()
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

pub fn find_invalid_variables<F>(
    domain: &IndexMap<String, DomainVariable>,
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
        .map(|((_, c), v)| c.to_i32().map(|c| v.times(c)))
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
        .map(|((_, c), v)| c.to_i32().map(|c| v.times(c)))
        .collect::<Option<Vec<_>>>()
}
