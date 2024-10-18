use crate::math::math_enums::{Comparison, OptimizationType, VariableType};
use crate::solvers::common::{Assignment, IntegerBinaryLpSolution, IntegerBinarySolverError};
use crate::transformers::linear_model::LinearModel;
use copper::views::{Times, ViewExt};
use copper::*;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Copy)]
#[serde(tag = "type", content = "value")]
pub enum VarValue {
    Bool(bool),
    Int(i32),
}

pub fn solve_integer_binary_lp_problem(
    lp: &LinearModel,
) -> Result<IntegerBinaryLpSolution<VarValue>, IntegerBinarySolverError> {
    let invalid_variables = lp
        .get_domain()
        .iter()
        .filter_map(|(name, var)| {
            if !matches!(
                var.get_type(),
                VariableType::Integer
                    | VariableType::Boolean
                    | VariableType::PositiveInteger
                    | VariableType::IntegerRange(_, _)
            ) {
                Some((name.clone(), var.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if !invalid_variables.is_empty() {
        return Err(IntegerBinarySolverError::InvalidDomain {
            expected: vec![
                VariableType::Integer,
                VariableType::Boolean,
                VariableType::PositiveInteger,
                VariableType::IntegerRange(i32::MIN, i32::MAX),
            ],
            got: invalid_variables,
        });
    }
    let binary_variables = lp
        .get_domain()
        .iter()
        .filter_map(|(name, var)| {
            if *var.get_type() == VariableType::Boolean {
                Some(name.clone())
            } else {
                None
            }
        })
        .enumerate()
        .map(|(i, name)| (name, i))
        .collect::<HashMap<_, _>>();
    let integer_variables = lp
        .get_domain()
        .iter()
        .filter_map(|(name, var)| {
            if matches!(
                var.get_type(),
                VariableType::Integer
                    | VariableType::IntegerRange(_, _)
                    | VariableType::PositiveInteger
            ) {
                Some(name.clone())
            } else {
                None
            }
        })
        .enumerate()
        .map(|(i, name)| (name, i))
        .collect::<HashMap<_, _>>();
    let mut m = Model::default();
    let vars_binary: Vec<_> = m.new_vars_binary(binary_variables.len()).collect();
    let vars_integer: Option<Vec<_>> = integer_variables
        .iter()
        .map(|(k, v)| {
            let domain = lp.get_domain().get(k).unwrap();
            let (min, max) = match domain.get_type() {
                VariableType::Integer => (-32768, 32768),
                VariableType::IntegerRange(min, max) => (*min, *max),
                VariableType::PositiveInteger => (0, 32768),
                _ => unreachable!(),
            };
            m.new_var(min, max)
        })
        .collect();
    let vars_integer = match vars_integer {
        Some(vars) => vars,
        None => {
            return Err(IntegerBinarySolverError::TooLarge {
                name: "integer variable".to_string(),
                value: i32::MAX as f64,
            })
        }
    };
    let vars = lp.get_variables();
    for (i, constraint) in lp.get_constraints().iter().enumerate() {
        let lhs_binary = process_variables_binary(
            constraint.get_coefficients().iter(),
            vars_binary.iter(),
            |i| binary_variables.get(&vars[i]).is_some(),
        );
        let lhs_integer = process_variables(
            constraint.get_coefficients().iter(),
            vars_integer.iter(),
            |i| integer_variables.get(&vars[i]).is_some(),
        );
        if lhs_binary.is_none() || lhs_integer.is_none() {
            return Err(IntegerBinarySolverError::TooLarge {
                name: format!("variable in constraint {}", i + 1),
                value: *constraint
                    .get_coefficients()
                    .iter()
                    .find(|c| c.to_f64().is_none())
                    .unwrap_or(&0.0),
            });
        }
        let lhs_binary = m.sum_iter(lhs_binary.unwrap());
        let lhs_integer = m.sum_iter(lhs_integer.unwrap());
        let lhs = m.sum(&[lhs_binary, lhs_integer]);
        let rhs = constraint.get_rhs().to_i32();
        if rhs.is_none() {
            return Err(IntegerBinarySolverError::TooLarge {
                name: format!("right hand side of constraint {}", i + 1),
                value: constraint.get_rhs(),
            });
        }
        match constraint.get_constraint_type() {
            Comparison::LowerOrEqual => {
                m.less_than_or_equals(lhs, rhs.unwrap());
            }
            Comparison::Equal => {
                m.equals(lhs, rhs.unwrap());
            }
            Comparison::UpperOrEqual => {
                m.greater_than_or_equals(lhs, rhs.unwrap());
            }
        }
    }
    let objective_binary =
        process_variables_binary(lp.get_objective().iter(), vars_binary.iter(), |i| {
            binary_variables.get(&vars[i]).is_some()
        });
    let objective_integer =
        process_variables(lp.get_objective().iter(), vars_integer.iter(), |i| {
            integer_variables.get(&vars[i]).is_some()
        });
    if objective_binary.is_none() || objective_integer.is_none() {
        return Err(IntegerBinarySolverError::TooLarge {
            name: "objective function variable".to_string(),
            value: *lp
                .get_objective()
                .iter()
                .find(|c| c.to_f64().is_none())
                .unwrap_or(&0.0),
        });
    }
    let objective_binary = m.sum_iter(objective_binary.unwrap());
    let objective_integer = m.sum_iter(objective_integer.unwrap());
    let objective = m.sum(&[objective_binary, objective_integer]);
    let solution = match lp.get_optimization_type() {
        OptimizationType::Max => m.maximize(objective),
        OptimizationType::Min => m.minimize(objective),
        OptimizationType::Satisfy => m.solve(),
    };

    let rev_binary_variables = binary_variables
        .iter()
        .map(|(name, i)| (i, name))
        .collect::<HashMap<_, _>>();
    let rev_integer_variables = integer_variables
        .iter()
        .map(|(name, i)| (i, name))
        .collect::<HashMap<_, _>>();
    match solution {
        None => Err(IntegerBinarySolverError::DidNotSolve),
        Some(solution) => {
            let assignment = solution
                .get_values_binary(&vars_binary)
                .iter()
                .enumerate()
                .map(|(v, n)| {
                    let name = *rev_binary_variables.get(&v).unwrap();
                    Assignment {
                        name: name.clone(),
                        value: VarValue::Bool(*n),
                    }
                })
                .chain(
                    solution
                        .get_values(&vars_integer)
                        .iter()
                        .enumerate()
                        .map(|(v, n)| {
                            let name = *rev_integer_variables.get(&v).unwrap();
                            Assignment {
                                name: name.clone(),
                                value: VarValue::Int(*n),
                            }
                        }),
                )
                .collect();

            let value = solution[objective] as f64 + lp.get_objective_offset();
            let sol = IntegerBinaryLpSolution::new(assignment, value);
            Ok(sol)
        }
    }
}

fn process_variables<'a, F>(
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

fn process_variables_binary<'a, F>(
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
