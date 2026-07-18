use crate::{LinearModel, LpSolution, MILPValue, SolverError, solve_milp_lp_problem};
use indexmap::IndexMap;

/// Solves any kind of linear programming problem with the built-in MILP solver.
///
/// The MILP solver (microlp) handles every supported variable type (boolean,
/// integer, real, non-negative real) and reports infeasible and unbounded models
/// correctly, so it is the safe default for `Auto`. The continuous Clarabel solver
/// is only used when it is selected explicitly ([`solve_real_lp_problem_clarabel`]
/// or the `Clarabel` builder solver).
///
/// [`solve_real_lp_problem_clarabel`]: crate::solve_real_lp_problem_clarabel
///
/// # Arguments
/// * `lp` - Any kind of linear programming model to solve
///
/// # Returns
/// * `Ok(LpSolution<MILPValue>)` - The optimal solution if found
/// * `Err(SolverError)` - Various error conditions that prevented finding a solution
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, auto_solver, LinearModel};
///
/// let mut model = LinearModel::new();
/// model.add_variable("x", VariableType::non_negative_real());
/// model.add_variable("y", VariableType::non_negative_real());
/// model.add_variable("z", VariableType::IntegerRange(0, 10));
///
/// // Machine time constraint: 3x + 2y + z <= 20
/// model.add_constraint(vec![3.0, 2.0, 1.0], Comparison::LessOrEqual, 20.0);
///
/// // Labor time constraint: 2x + y + 3z <= 15
/// model.add_constraint(vec![2.0, 1.0, 3.0], Comparison::LessOrEqual, 15.0);
///
/// // Minimum production constraint for x: x >= 2
/// model.add_constraint(vec![1.0, 0.0, 0.0], Comparison::GreaterOrEqual, 2.0);
///
/// // Maximum production constraint for y: y <= 7
/// model.add_constraint(vec![0.0, 1.0, 0.0], Comparison::LessOrEqual, 7.0);
///
/// // Set objective: maximize 50x + 40y + 45z
/// model.set_objective(vec![50.0, 40.0, 45.0], OptimizationType::Max);
///
/// let solution = auto_solver(&model).unwrap();
/// ```
pub fn auto_solver(lp: &LinearModel) -> Result<LpSolution<MILPValue>, SolverError> {
    if lp.domain().is_empty() {
        // A variable-free model still carries a constant objective (the offset).
        return Ok(LpSolution::new(
            vec![],
            lp.objective_offset(),
            IndexMap::new(),
        ));
    }
    solve_milp_lp_problem(lp)
}
