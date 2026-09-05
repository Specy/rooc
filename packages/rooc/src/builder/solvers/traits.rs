//! Solver and solution traits shared across the builder.

use crate::solvers::{LpSolution, SolutionStatus, SolveOutcome, SolverError, TerminationReason};
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
    fn solve(&self, model: &LinearModel) -> Result<SolveOutcome<Self::Solution>, SolverError>;
}

/// The core of every solution: the objective value and per-variable values.
pub trait Solution {
    /// The value type of a variable (e.g. `MILPValue` or `f64`).
    type Value: Copy + Into<f64>;

    /// The objective value reported for this solution.
    fn objective_value(&self) -> f64;

    /// The value of a variable by its (linear-model) name.
    fn var_value(&self, variable: &str) -> Option<Self::Value>;
}

/// Optional capability: the reported solve status.
pub trait SolveStatus {
    fn status(&self) -> SolutionStatus;

    /// Why the search that produced this solution stopped.
    fn termination_reason(&self) -> TerminationReason;

    /// The best objective bound proven by the search, when the backend reports
    /// one. Defaults to `None` for backends that track no bound.
    fn best_bound(&self) -> Option<f64> {
        None
    }

    /// The relative gap between this solution and the best bound, when the
    /// backend reports one.
    fn gap(&self) -> Option<f64> {
        None
    }
}

/// Optional capability: the activity (row value) of a constraint at the solution.
pub trait ConstraintValues {
    fn constraint_value(&self, constraint: &str) -> Option<f64>;
}

/// Optional capability: dual values (shadow prices), keyed by constraint name.
///
/// Implementors must report the rate of change of the objective with respect to the
/// constraint's right-hand side: raising the RHS of a binding `x >= 2` in a `min x`
/// model by one raises the objective by one, so its shadow price is `+1`. Backends
/// whose solver normalizes the row (negating a `>=` into a `<=`, say) have to undo
/// that before reporting, or callers get a sign that depends on the solver.
pub trait DualValues {
    fn shadow_price(&self, constraint: &str) -> Option<f64>;
}

/// Optional capability: reduced costs, keyed by variable name.
pub trait ReducedCosts {
    fn reduced_cost(&self, variable: &str) -> Option<f64>;
}

// The built-in solvers use `LpSolution` as their solution type. It provides the
// core `Solution`, `SolveStatus`, and `ConstraintValues`. Backends that expose
// duals populate its optional shadow-price map; other backends return `None`
// through `DualValues`. No backend populates reduced costs yet.
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

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned + Copy + std::fmt::Display>
    SolveStatus for LpSolution<T>
{
    fn status(&self) -> SolutionStatus {
        // Explicit path resolves to the inherent accessor, not this trait method.
        LpSolution::status(self)
    }

    fn termination_reason(&self) -> TerminationReason {
        LpSolution::termination_reason(self)
    }

    fn best_bound(&self) -> Option<f64> {
        LpSolution::best_bound(self)
    }

    fn gap(&self) -> Option<f64> {
        LpSolution::gap(self)
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned + Copy + std::fmt::Display>
    ConstraintValues for LpSolution<T>
{
    fn constraint_value(&self, constraint: &str) -> Option<f64> {
        self.constraints().get(constraint).copied()
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned + Copy + std::fmt::Display>
    DualValues for LpSolution<T>
{
    fn shadow_price(&self, constraint: &str) -> Option<f64> {
        self.shadow_prices().get(constraint).copied()
    }
}
