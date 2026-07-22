//! Clarabel backend provided by `good_lp`.

use super::good_lp::{collect_good_lp_duals, solve_with_good_lp};
use super::{LpSolution, SolverError, find_invalid_variables};
use crate::math::VariableType;
use crate::transformers::LinearModel;
use ::clarabel::solver::SolverStatus;
use ::good_lp::SolutionWithDual;

/// Solves a linear programming problem with real variables using the Clarabel solver.
///
/// Takes a linear model containing real or non-negative real variables and returns an optimal solution
/// or an error if the problem cannot be solved.
///
/// # Arguments
/// * `lp` - The linear programming model to solve, must contain only real or non-negative real variables
///
/// # Returns
/// * `Ok(LpSolution<f64>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, solve_real_lp_problem_clarabel, LinearModel};
///
/// let mut model = LinearModel::new();
/// model.add_variable("x1", VariableType::non_negative_real());
/// model.add_variable("x2", VariableType::real());
///
/// // Add constraint: x1 + x2 <= 5
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 5.0);
///
/// // Set objective: maximize x1 + 2*x2
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Max);
///
/// let solution = solve_real_lp_problem_clarabel(&model).unwrap();
/// ```
pub fn solve_real_lp_problem_clarabel(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    let domain = lp.domain();
    let invalid_variables = find_invalid_variables(domain, |var| {
        matches!(
            var,
            VariableType::Real(_, _) | VariableType::NonNegativeReal(_, _)
        )
    });
    if !invalid_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![
                VariableType::Real(f64::NEG_INFINITY, f64::INFINITY),
                VariableType::NonNegativeReal(0.0, f64::INFINITY),
            ],
            got: invalid_variables,
        });
    }
    solve_with_good_lp(
        lp,
        ::good_lp::clarabel,
        |model, _| Ok(model),
        |solution| {
            // good_lp maps clarabel's (Almost)DualInfeasible status to `Ok`, but for
            // an LP a dual-infeasible problem is a certificate that the primal is
            // unbounded. Inspect the underlying clarabel status and report it, rather
            // than returning the meaningless point clarabel hands back.
            if matches!(
                solution.inner().status,
                SolverStatus::DualInfeasible | SolverStatus::AlmostDualInfeasible
            ) {
                return Err(SolverError::Unbounded);
            }
            Ok(())
        },
        |solution, references| {
            let dual = solution.compute_dual();
            collect_good_lp_duals(dual, references)
        },
    )
}
