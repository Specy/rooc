use crate::solvers::common::{
    DisplayValue, InterruptedSolve, LpSolution, SolutionStatus, SolveOutcome, SolverError,
    TerminationReason, format_float,
};
use crate::transformers::LinearModel;
use crate::{
    Assignment, Comparison, OptimizationType, VariableType, make_constraints_map_from_assignment,
};
use microlp::{
    ComparisonOp, Error, OptimizationDirection, Problem, SolutionStatus as MicrolpSolutionStatus,
    SolveOptions, SolveOutcome as MicrolpSolveOutcome, TerminationReason as MicrolpTermination,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;

/// Represents a variable value that can be either boolean or integer.
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(tag = "type", content = "value")]
pub enum MILPValue {
    /// A boolean value (true/false)
    Bool(bool),
    /// An integer value
    Int(i32),
    /// A real value
    Real(f64),
}
impl Display for MILPValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MILPValue::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            MILPValue::Int(i) => write!(f, "{}", i),
            MILPValue::Real(r) => write!(f, "{}", format_float(*r)),
        }
    }
}

impl DisplayValue for MILPValue {
    fn display_value(&self) -> String {
        self.to_string()
    }
}

impl From<MILPValue> for f64 {
    fn from(value: MILPValue) -> f64 {
        match value {
            MILPValue::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            MILPValue::Int(i) => i as f64,
            MILPValue::Real(r) => r,
        }
    }
}

/// Tunable parameters for the MicroLP mixed-integer solver.
///
/// The default is an empty configuration, which reproduces the behaviour of
/// [`solve_milp_lp_problem`].
#[derive(Debug, Clone, Default)]
pub struct MilpOptions {
    /// Relative MIP gap at which the branch-and-bound search may stop early.
    /// Must be a non-negative, finite number when set.
    pub mip_gap: Option<f64>,
    /// Wall-clock limit for the search.
    pub time_limit: Option<Duration>,
    /// Maximum number of branch-and-bound nodes to explore. A deterministic
    /// alternative to `time_limit`; the root relaxation does not count as a
    /// node. It has no effect on a model with no integer or boolean variables,
    /// because such a model is solved without branching.
    pub node_limit: Option<u64>,
}

/// Translates a MicroLP termination reason into the crate's own.
///
/// MicroLP's variants map one-to-one; `IterationLimit` is never produced here
/// because MicroLP counts nodes rather than pivots.
fn map_termination(reason: MicrolpTermination) -> TerminationReason {
    match reason {
        MicrolpTermination::ProvenOptimal => TerminationReason::ProvenOptimal,
        MicrolpTermination::MipGap => TerminationReason::MipGap,
        MicrolpTermination::TimeLimit => TerminationReason::TimeLimit,
        MicrolpTermination::NodeLimit => TerminationReason::NodeLimit,
        // `TerminationReason` is `#[non_exhaustive]` upstream, so an unknown
        // future variant degrades to the closest safe meaning: the search
        // stopped without a completed proof.
        _ => TerminationReason::TimeLimit,
    }
}
/// Solves a mixed-integer linear programming problem using the MicroLP solver.
///
/// Takes a linear model containing real, non-negative real, boolean, and integer variables and returns
/// an optimal solution or an error if the problem cannot be solved.
///
/// # Arguments
/// * `lp` - The mixed-integer linear programming model to solve
///
/// # Returns
/// * `Ok(SolveOutcome::Solution)` - A usable assignment, optimal or feasible
/// * `Ok(SolveOutcome::Interrupted)` - A limit stopped the search before any
///   assignment was found. This is not a failure: the model may still be
///   solvable given a larger budget
/// * `Err(SolverError)` - Conditions that prevent a solution existing at all,
///   such as an infeasible or unbounded model
///
/// # Example
/// ```rust
/// use rooc::{VariableType, Comparison, OptimizationType, solve_milp_lp_problem, LinearModel};
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
/// // With no limits configured the search always runs to proven optimality,
/// // so the outcome is guaranteed to hold a solution.
/// let solution = solve_milp_lp_problem(&model).unwrap().into_solution().unwrap();
/// ```
pub fn solve_milp_lp_problem(
    lp: &LinearModel,
) -> Result<SolveOutcome<LpSolution<MILPValue>>, SolverError> {
    solve_milp_lp_problem_with(lp, &MilpOptions::default())
}

/// Like [`solve_milp_lp_problem`], but with explicit control over the solver
/// through [`MilpOptions`] (MIP gap, time limit, node limit).
pub fn solve_milp_lp_problem_with(
    lp: &LinearModel,
    options: &MilpOptions,
) -> Result<SolveOutcome<LpSolution<MILPValue>>, SolverError> {
    let variables = lp.variables();
    let domain = lp.domain();
    let objective = lp.objective();
    if objective.len() != variables.len() {
        return Err(SolverError::Other(format!(
            "objective length {} does not match variable count {}",
            objective.len(),
            variables.len()
        )));
    }
    let mut microlp_vars = Vec::with_capacity(variables.len());
    let opt_type = match lp.optimization_type() {
        OptimizationType::Max => OptimizationDirection::Maximize,
        OptimizationType::Min => OptimizationDirection::Minimize,
        OptimizationType::Satisfy => OptimizationDirection::Minimize,
    };
    let mut problem = Problem::new(opt_type);
    for (i, var) in variables.iter().enumerate() {
        let var_domain = domain.get(var).unwrap();
        let coeff = objective[i];
        let added_var = match var_domain.get_type() {
            VariableType::Real(min, max) => problem.add_var(coeff, (*min, *max)),
            VariableType::Boolean => problem.add_binary_var(coeff),
            VariableType::IntegerRange(min, max) => problem.add_integer_var(coeff, (*min, *max)),
            VariableType::NonNegativeReal(min, max) => problem.add_var(coeff, (*min, *max)),
        };
        microlp_vars.push(added_var);
    }

    for constraint in lp.constraints() {
        let coeffs = constraint.coefficients();
        let rhs = constraint.rhs();
        let comparison_type = constraint.constraint_type();
        let microlp_comparison_type = match comparison_type {
            Comparison::LessOrEqual => ComparisonOp::Le,
            Comparison::GreaterOrEqual => ComparisonOp::Ge,
            Comparison::Equal => ComparisonOp::Eq,
            c => {
                return Err(SolverError::UnavailableComparison {
                    got: *c,
                    expected: vec![
                        Comparison::LessOrEqual,
                        Comparison::GreaterOrEqual,
                        Comparison::Equal,
                    ],
                });
            }
        };
        let microlp_coeffs = microlp_vars
            .iter()
            .zip(coeffs.iter())
            .map(|(v, c)| (*v, *c))
            .collect::<Vec<_>>();
        problem.add_constraint(microlp_coeffs, microlp_comparison_type, rhs);
    }

    let mut solve_options = SolveOptions::default();
    if let Some(gap) = options.mip_gap {
        // MicroLP validates the gap (finite and non-negative) and returns a
        // clear error, so it is forwarded directly rather than re-checked here.
        solve_options.mip_gap = gap;
    }
    if let Some(limit) = options.time_limit {
        solve_options.time_limit = Some(limit);
    }
    if let Some(limit) = options.node_limit {
        solve_options.node_limit = Some(limit);
    }

    match problem.solve_with(solve_options) {
        Ok(MicrolpSolveOutcome::Solution(solution)) => {
            let status = match solution.status() {
                MicrolpSolutionStatus::Optimal => SolutionStatus::Optimal,
                MicrolpSolutionStatus::Feasible => SolutionStatus::Feasible,
            };
            let reason = map_termination(solution.termination_reason());
            let stats = solution.stats();
            // MicroLP reports the bound on its own objective, so the model's
            // constant offset is added to keep it comparable with `value()`.
            // The gap is relative and measured on that same un-offset
            // objective, so it is passed through untouched.
            let best_bound = stats.best_bound.map(|bound| bound + lp.objective_offset());
            let gap = stats.gap;
            let assignment = microlp_vars
                .iter()
                .zip(variables)
                .map(|(v, name)| {
                    let value = solution.var_value(*v);
                    let var_domain = domain.get(name).unwrap();
                    let value = match var_domain.get_type() {
                        VariableType::Real(_, _) | VariableType::NonNegativeReal(_, _) => {
                            MILPValue::Real(value)
                        }
                        VariableType::IntegerRange(_, _) => MILPValue::Int(value as i32),
                        VariableType::Boolean => MILPValue::Bool(value != 0.0),
                    };
                    Assignment {
                        name: name.clone(),
                        value,
                    }
                })
                .collect();
            let coeffs = microlp_vars
                .iter()
                .map(|v| solution.var_value(*v))
                .collect();
            let constraints = make_constraints_map_from_assignment(lp, &coeffs);
            Ok(SolveOutcome::Solution(
                LpSolution::new(
                    assignment,
                    solution.objective() + lp.objective_offset(),
                    constraints,
                )
                .with_status(status)
                .with_termination_reason(reason)
                .with_best_bound(best_bound)
                .with_gap(gap),
            ))
        }
        Ok(MicrolpSolveOutcome::Interrupted(interrupted)) => {
            // A limit fired before any assignment was found. Not an error: the
            // model may still be solvable with a larger budget.
            let stats = interrupted.stats();
            Ok(SolveOutcome::Interrupted(
                InterruptedSolve::new(map_termination(interrupted.termination_reason()))
                    .with_best_bound(stats.best_bound.map(|bound| bound + lp.objective_offset()))
                    .with_gap(stats.gap),
            ))
        }
        Err(e) => Err(match e {
            Error::InternalError(s) => SolverError::Other(s),
            Error::Unbounded => SolverError::Unbounded,
            Error::InvalidOptions(s) => SolverError::Other(s),
            Error::InvalidOperation(s) => SolverError::Other(s),
            Error::Infeasible => SolverError::Infeasible,
        }),
    }
}
