use crate::math::math_enums::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::transformer_context::DomainVariable;
use crate::transformers::linear_model::LinearModel;
use copper::views::ViewExt;
use copper::*;
use num_traits::ToPrimitive;
use serde::Serialize;

#[derive(Debug)]
pub enum BinarySolverError {
    InvalidDomain(Vec<(String, DomainVariable)>),
    TooLarge { name: String, value: f64 },
    DidNotSolve,
}

impl std::fmt::Display for BinarySolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinarySolverError::InvalidDomain(vars) => {
                let vars = vars
                    .iter()
                    .map(|(name, domain)| format!("    {}: {}", name, domain.get_type()))
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(f, "Invalid domain, the following variables are non binary: \n{}", vars)
            }
            BinarySolverError::TooLarge { name, value } => {
                write!(f, "The value of variable {} is too large: {}", name, value)
            }
            BinarySolverError::DidNotSolve => {
                write!(f, "The problem was able to be solved")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Assignment {
    pub name: String,
    pub value: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BinaryLpSolution {
    assignment: Vec<Assignment>,
    value: f64,
}

impl BinaryLpSolution {
    pub fn get_assignment(&self) -> &Vec<Assignment> {
        &self.assignment
    }
    pub fn get_assignment_values(&self) -> Vec<bool> {
        self.assignment.iter().map(|a| a.value).collect()
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub fn solve_binary_lp_problem(lp: &LinearModel) -> Result<BinaryLpSolution, BinarySolverError> {
    let non_binary_variables = lp
        .get_domain()
        .iter()
        .filter_map(|(name, var)| {
            if *var.get_type() != VariableType::Boolean {
                Some((name.clone(), var.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if non_binary_variables.len() > 0 {
        return Err(BinarySolverError::InvalidDomain(non_binary_variables));
    }
    let mut m = Model::default();
    let vars: Vec<_> = m.new_vars_binary(lp.get_domain().len()).collect();

    for (i, constraint) in lp.get_constraints().iter().enumerate() {
        let lhs = constraint
            .get_coefficients()
            .iter()
            .zip(vars.iter())
            .map(|(c, v)| c.to_i32().map(|c| v.times(c)))
            .collect::<Option<Vec<_>>>();
        if lhs.is_none() {
            return Err(BinarySolverError::TooLarge {
                name: format!("variable in constraint {}", i + 1),
                value: *constraint
                    .get_coefficients()
                    .iter()
                    .find(|c| c.to_f64().is_none())
                    .unwrap_or(&0.0),
            });
        }
        let lhs = m.sum_iter(lhs.unwrap());
        let rhs = constraint.get_rhs().to_i32();
        if rhs.is_none() {
            return Err(BinarySolverError::TooLarge {
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
    let objective = lp
        .get_objective()
        .iter()
        .zip(vars.iter())
        .map(|(c, v)| c.to_i32().map(|c| v.times(c)))
        .collect::<Option<Vec<_>>>();
    if objective.is_none() {
        return Err(BinarySolverError::TooLarge {
            name: "objective function variable".to_string(),
            value: *lp
                .get_objective()
                .iter()
                .find(|c| c.to_f64().is_none())
                .unwrap_or(&0.0),
        });
    }
    let objective = m.sum_iter(objective.unwrap());
    let solution = match lp.get_optimization_type() {
        OptimizationType::Max => m.maximize(objective),
        OptimizationType::Min => m.minimize(objective),
    };
    match solution {
        None => {
            return Err(BinarySolverError::DidNotSolve);
        }
        Some(solution) => {
            let var_names = lp.get_variables();
            let assignment = solution
                .get_values_binary(&vars)
                .iter()
                .zip(var_names.iter())
                .map(|(v, n)| Assignment {
                    name: n.clone(),
                    value: *v,
                })
                .collect();
            let value = solution[objective];
            Ok(BinaryLpSolution {
                assignment,
                value: value.to_f64().unwrap() + lp.get_objective_offset(),
            })
        }
    }
}
