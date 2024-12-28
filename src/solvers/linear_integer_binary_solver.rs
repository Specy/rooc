use crate::math::{Comparison, OptimizationType, VariableType};
use crate::solvers::common::{
    find_invalid_variables, process_variables, process_variables_binary, Assignment, LpSolution,
    SolverError,
};
use crate::transformers::LinearModel;
use copper::*;
use indexmap::IndexMap;
use num_traits::ToPrimitive;
use serde::Serialize;
use std::fmt::{Display, Formatter};

/// Represents a variable value that can be either boolean or integer.
#[derive(Debug, Clone, Serialize, Copy)]
#[serde(tag = "type", content = "value")]
pub enum IntOrBoolValue {
    /// A boolean value (true/false)
    Bool(bool),
    /// An integer value
    Int(i32),
}
impl Display for IntOrBoolValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IntOrBoolValue::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            IntOrBoolValue::Int(i) => write!(f, "{}", i),
        }
    }
}

/// Solves a mixed integer-binary linear programming problem.
///
/// Takes a linear model containing boolean and integer variables and returns an optimal solution
/// or an error if the problem cannot be solved.
/// All coefficients and constant numbers will be converted into an i32
///
/// # Arguments
/// * `lp` - The linear programming model to solve
///
/// # Returns
/// * `Ok(LpSolution<VarValue>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, solve_integer_binary_lp_problem, LinearModel};
///
/// let mut model = LinearModel::new();
///
/// // Add a boolean variable x1 and an integer variable x2
/// model.add_variable("x1", VariableType::Boolean);
/// model.add_variable("x2", VariableType::IntegerRange(0, 10));
///
/// // Add constraint: x1 + x2 <= 5
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0);
///
/// // Set objective: maximize x1 + 2*x2
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Max);
///
/// let solution = solve_integer_binary_lp_problem(&model).unwrap();
/// ```
pub fn solve_integer_binary_lp_problem(
    lp: &LinearModel,
) -> Result<LpSolution<IntOrBoolValue>, SolverError> {
    let invalid_variables = find_invalid_variables(lp.domain(), |var| {
        matches!(
            var,
            VariableType::Boolean | VariableType::IntegerRange(_, _)
        )
    });
    if !invalid_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![
                VariableType::Boolean,
                VariableType::IntegerRange(i32::MIN, i32::MAX),
            ],
            got: invalid_variables,
        });
    }
    let domain = lp.domain();
    let vars = lp.variables().clone();
    let enum_vars: Vec<(usize, String)> = vars.into_iter().enumerate().collect();
    let binary_variables = enum_vars
        .iter()
        .filter_map(|(indx, var)| {
            let domain = domain.get(var).unwrap();
            if *domain.get_type() == VariableType::Boolean {
                Some((var.clone(), *indx))
            } else {
                None
            }
        })
        .collect::<IndexMap<_, _>>();
    let integer_variables = enum_vars
        .iter()
        .filter_map(|(indx, var)| {
            let domain = domain.get(var).unwrap();
            if matches!(domain.get_type(), VariableType::IntegerRange(_, _)) {
                Some((var.clone(), *indx))
            } else {
                None
            }
        })
        .collect::<IndexMap<_, _>>();

    let mut m = Model::default();
    let vars_binary: Vec<_> = m.new_vars_binary(binary_variables.len()).collect();
    let vars_integer: Option<Vec<_>> = integer_variables
        .iter()
        .map(|(k, _)| {
            let domain = lp.domain().get(k).unwrap();
            let (min, max) = match domain.get_type() {
                VariableType::IntegerRange(min, max) => (*min, *max),
                _ => unreachable!(),
            };
            m.new_var(min, max)
        })
        .collect();
    let vars_integer = match vars_integer {
        Some(vars) => vars,
        None => {
            return Err(SolverError::TooLarge {
                name: "integer variable".to_string(),
                value: i32::MAX as f64,
            })
        }
    };
    let vars = lp.variables();
    for (i, constraint) in lp.constraints().iter().enumerate() {
        let lhs_binary =
            process_variables_binary(constraint.coefficients().iter(), vars_binary.iter(), |i| {
                binary_variables.get(&vars[i]).is_some()
            });
        let lhs_integer =
            process_variables(constraint.coefficients().iter(), vars_integer.iter(), |i| {
                integer_variables.get(&vars[i]).is_some()
            });
        if lhs_binary.is_none() || lhs_integer.is_none() {
            return Err(SolverError::TooLarge {
                name: format!("variable in constraint {}", i + 1),
                value: *constraint
                    .coefficients()
                    .iter()
                    .find(|c| c.to_f64().is_none())
                    .unwrap_or(&0.0),
            });
        }
        let mut exprs = vec![];
        let lhs_binary = lhs_binary.unwrap();
        let lhs_integer = lhs_integer.unwrap();
        if !lhs_binary.is_empty() {
            exprs.push(m.sum_iter(lhs_binary));
        }
        if !lhs_integer.is_empty() {
            exprs.push(m.sum_iter(lhs_integer));
        }
        //don't do unnecessary sums if there are no integer or no binary variables
        let lhs = match exprs.len() {
            1 => exprs[0],
            _ => m.sum(&exprs),
        };
        let rhs = constraint.rhs().to_i32();
        if rhs.is_none() {
            return Err(SolverError::TooLarge {
                name: format!("right hand side of constraint {}", i + 1),
                value: constraint.rhs(),
            });
        }
        let rhs = rhs.unwrap();
        match constraint.constraint_type() {
            Comparison::LessOrEqual => {
                m.less_than_or_equals(lhs, rhs);
            }
            Comparison::Equal => {
                m.equals(lhs, rhs);
            }
            Comparison::GreaterOrEqual => {
                m.greater_than_or_equals(lhs, rhs);
            }
            Comparison::Less => {
                m.less_than(lhs, rhs);
            }
            Comparison::Greater => {
                m.greater_than(lhs, rhs);
            }
        }
    }
    let objective_binary =
        process_variables_binary(lp.objective().iter(), vars_binary.iter(), |i| {
            binary_variables.get(&vars[i]).is_some()
        });
    let objective_integer = process_variables(lp.objective().iter(), vars_integer.iter(), |i| {
        integer_variables.get(&vars[i]).is_some()
    });
    if objective_binary.is_none() || objective_integer.is_none() {
        return Err(SolverError::TooLarge {
            name: "objective function variable".to_string(),
            value: *lp
                .objective()
                .iter()
                .find(|c| c.to_f64().is_none())
                .unwrap_or(&0.0),
        });
    }
    let objective_binary = objective_binary.unwrap();
    let objective_integer = objective_integer.unwrap();

    let mut obj_exprs = vec![];
    if !objective_binary.is_empty() {
        obj_exprs.push(m.sum_iter(objective_binary));
    }
    if !objective_integer.is_empty() {
        obj_exprs.push(m.sum_iter(objective_integer));
    }
    //don't do unnecessary sums if there are no integer or no binary variables
    let objective = match obj_exprs.len() {
        1 => obj_exprs[0],
        _ => m.sum(&obj_exprs),
    };

    let solution = match lp.optimization_type() {
        OptimizationType::Max => m.maximize(objective),
        OptimizationType::Min => m.minimize(objective),
        OptimizationType::Satisfy => m.solve(),
    };

    let rev_binary_variables = binary_variables
        .iter()
        .map(|(name, i)| (i, name))
        .collect::<IndexMap<_, _>>();
    let rev_integer_variables = integer_variables
        .iter()
        .map(|(name, i)| (i, name))
        .collect::<IndexMap<_, _>>();

    match solution {
        None => Err(SolverError::DidNotSolve),
        Some(solution) => {
            let mut assignment = solution
                .get_values_binary(&vars_binary)
                .iter()
                .enumerate()
                .filter_map(|(v, n)| {
                    let name = rev_binary_variables.get(&v);
                    name.map(|name| Assignment {
                        name: (*name).clone(),
                        value: IntOrBoolValue::Bool(*n),
                    })
                })
                .chain(
                    solution
                        .get_values(&vars_integer)
                        .iter()
                        .enumerate()
                        .filter_map(|(v, n)| {
                            let name = rev_integer_variables.get(&v);
                            name.map(|name| Assignment {
                                name: (*name).clone(),
                                value: IntOrBoolValue::Int(*n),
                            })
                        }),
                )
                .collect::<Vec<Assignment<IntOrBoolValue>>>();
            assignment.sort_by(|a, b| a.name.cmp(&b.name));
            let value = solution[objective] as f64 + lp.objective_offset();
            let sol = LpSolution::new(assignment, value);
            Ok(sol)
        }
    }
}
