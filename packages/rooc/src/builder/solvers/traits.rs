//! Solver and solution traits shared across the builder.

use crate::solvers::{LpSolution, SolutionStatus, SolverError};
use crate::transformers::linear_model::LinearModel;

/// A solver applied to a linearized model produced by the builder.
///
/// Each solver returns its own [`Solution`] type, so a solution exposes exactly
/// the capabilities its solver supports. Implement this (and optionally the
/// capability traits below) to add a back-end.
pub trait Solver {
    /// The solution type this solver produces.
    type Solution: Solution;

    /// Solves the given linearized model.
    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError>;
}

/// The core of every solution: the objective value and per-variable values.
pub trait Solution {
    /// The value type of a variable (e.g. `MILPValue` or `f64`).
    type Value: Copy + Into<f64>;

    /// The optimal objective value.
    fn objective_value(&self) -> f64;

    /// The value of a variable by its (linear-model) name.
    fn var_value(&self, variable: &str) -> Option<Self::Value>;
}

/// Optional capability: the reported solve status.
pub trait SolveStatus {
    fn status(&self) -> SolutionStatus;
}

/// Optional capability: the activity (row value) of a constraint at the solution.
pub trait ConstraintValues {
    fn constraint_value(&self, constraint: &str) -> Option<f64>;
}

/// Optional capability: dual values (shadow prices), keyed by constraint name.
pub trait DualValues {
    fn shadow_price(&self, constraint: &str) -> Option<f64>;
}

/// Optional capability: reduced costs, keyed by variable name.
pub trait ReducedCosts {
    fn reduced_cost(&self, variable: &str) -> Option<f64>;
}

// The built-in solvers use `LpSolution` as their solution type. It provides the
// core `Solution` plus `SolveStatus` and `ConstraintValues`, but deliberately
// not `DualValues`/`ReducedCosts`, so those methods do not appear on built-in
// solutions.
impl<
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + Copy + std::fmt::Display + Into<f64>,
> Solution for LpSolution<T>
{
    type Value = T;

    fn objective_value(&self) -> f64 {
        self.value()
    }

    fn var_value(&self, variable: &str) -> Option<T> {
        self.value_of(variable)
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned + Copy + std::fmt::Display> SolveStatus
    for LpSolution<T>
{
    fn status(&self) -> SolutionStatus {
        // Explicit path resolves to the inherent accessor, not this trait method.
        LpSolution::status(self)
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned + Copy + std::fmt::Display>
    ConstraintValues for LpSolution<T>
{
    fn constraint_value(&self, constraint: &str) -> Option<f64> {
        self.constraints().get(constraint).copied()
    }
}
