use crate::math::math_enums::{Comparison, OptimizationType, VariableType};
use crate::solvers::common::{
    find_invalid_variables, Assignment, IntegerBinaryLpSolution, SolverError,
};
use crate::transformers::linear_model::LinearModel;
use copper::views::ViewExt;
use copper::*;
use num_traits::ToPrimitive;

pub fn solve_binary_lp_problem(
    lp: &LinearModel,
) -> Result<IntegerBinaryLpSolution<bool>, SolverError> {
    let non_binary_variables =
        find_invalid_variables(lp.get_domain(), |var| matches!(var, VariableType::Boolean));
    if !non_binary_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![VariableType::Boolean],
            got: non_binary_variables,
        });
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
            return Err(SolverError::TooLarge {
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
            return Err(SolverError::TooLarge {
                name: format!("right hand side of constraint {}", i + 1),
                value: constraint.get_rhs(),
            });
        }
        match constraint.get_constraint_type() {
            Comparison::LessOrEqual => {
                m.less_than_or_equals(lhs, rhs.unwrap());
            }
            Comparison::Equal => {
                m.equals(lhs, rhs.unwrap());
            }
            Comparison::GreaterOrEqual => {
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
        return Err(SolverError::TooLarge {
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
        OptimizationType::Satisfy => m.solve(),
    };
    match solution {
        None => Err(SolverError::DidNotSolve),
        Some(solution) => {
            let var_names = lp.get_variables();
            let mut assignment = solution
                .get_values_binary(&vars)
                .iter()
                .zip(var_names.iter())
                .map(|(v, n)| Assignment {
                    name: n.clone(),
                    value: *v,
                })
                .collect::<Vec<Assignment<bool>>>();
            let value = solution[objective] as f64 + lp.get_objective_offset();
            assignment.sort_by(|a, b| a.name.cmp(&b.name));
            let sol = IntegerBinaryLpSolution::new(assignment, value);
            Ok(sol)
        }
    }
}
