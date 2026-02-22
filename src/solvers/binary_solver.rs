use crate::make_constraints_map_from_assignment;
use crate::math::{Comparison, OptimizationType, VariableType};
use crate::solvers::common::{Assignment, LpSolution, SolverError, find_invalid_variables};
use crate::transformers::LinearModel;
use copper::views::ViewExt;
use copper::*;
use num_traits::ToPrimitive;

/// Solves a binary linear programming problem.
///
/// Takes a linear model containing only boolean variables and returns an optimal solution
/// or an error if the problem cannot be solved.
///
/// # Arguments
/// * `lp` - The linear programming model to solve, must contain only boolean variables
///
/// # Returns
/// * `Ok(LpSolution<bool>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, solve_binary_lp_problem, LinearModel};
///
/// let mut model = LinearModel::new();
/// model.add_variable("x1", VariableType::Boolean);
/// model.add_variable("x2", VariableType::Boolean);
///
/// // Add constraint: x1 + x2 <= 1
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 1.0);
///
/// // Set objective: maximize x1 + 2*x2
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Max);
///
/// let solution = solve_binary_lp_problem(&model).unwrap();
/// ```
pub fn solve_binary_lp_problem(lp: &LinearModel) -> Result<LpSolution<bool>, SolverError> {
    let non_binary_variables =
        find_invalid_variables(lp.domain(), |var| matches!(var, VariableType::Boolean));
    if !non_binary_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![VariableType::Boolean],
            got: non_binary_variables,
        });
    }
    let mut m = Model::default();
    let vars: Vec<_> = m.new_vars_binary(lp.domain().len()).collect();

    for (i, constraint) in lp.constraints().iter().enumerate() {
        let lhs = constraint
            .coefficients()
            .iter()
            .zip(vars.iter())
            .map(|(c, v)| c.to_i32().map(|c| v.times(c)))
            .collect::<Option<Vec<_>>>();
        if lhs.is_none() {
            return Err(SolverError::TooLarge {
                name: format!("variable in constraint {}", i + 1),
                value: *constraint
                    .coefficients()
                    .iter()
                    .find(|c| c.to_f64().is_none())
                    .unwrap_or(&0.0),
            });
        }
        let lhs = m.sum_iter(lhs.unwrap());
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
    let objective = lp
        .objective()
        .iter()
        .zip(vars.iter())
        .map(|(c, v)| c.to_i32().map(|c| v.times(c)))
        .collect::<Option<Vec<_>>>();
    if objective.is_none() {
        return Err(SolverError::TooLarge {
            name: "objective function variable".to_string(),
            value: *lp
                .objective()
                .iter()
                .find(|c| c.to_f64().is_none())
                .unwrap_or(&0.0),
        });
    }
    let objective = m.sum_iter(objective.unwrap());
    let solution = match lp.optimization_type() {
        OptimizationType::Max => m.maximize(objective),
        OptimizationType::Min => m.minimize(objective),
        OptimizationType::Satisfy => m.solve(),
    };
    match solution {
        None => Err(SolverError::DidNotSolve),
        Some(solution) => {
            let var_names = lp.variables();
            let mut assignment = solution
                .get_values_binary(&vars)
                .iter()
                .zip(var_names.iter())
                .map(|(v, n)| Assignment {
                    name: n.clone(),
                    value: *v,
                })
                .collect::<Vec<Assignment<bool>>>();
            let coeffs = assignment
                .iter()
                .map(|v| if v.value { 1.0 } else { 0.0 })
                .collect();
            let constraints = make_constraints_map_from_assignment(lp, &coeffs);

            let value = solution[objective] as f64 + lp.objective_offset();
            assignment.sort_by(|a, b| a.name.cmp(&b.name));
            let sol = LpSolution::new(assignment, value, constraints);
            Ok(sol)
        }
    }
}
