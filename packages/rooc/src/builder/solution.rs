//! The builder's solution wrapper and capability forwarding.

use super::expr::{Expr, Var, eval_expr};
use super::solvers::{ConstraintValues, DualValues, ReducedCosts, Solution, SolveStatus, Solver};
use crate::solvers::{SolutionStatus, TerminationReason};

/// A solution produced by [`crate::ModelBuilder::solve_with`].
///
/// Wraps the solver's own [`Solution`] and lets you read values back through the
/// [`Var`] handles minted by the builder. Optional capabilities
/// (`status`, `constraint_value`, `shadow_price`, `reduced_cost`) are available
/// only when the solver's solution implements the matching trait. A solved
/// builder model is immutable; add every constraint before solving it.
pub struct BuilderSolution<S: Solver> {
    solution: S::Solution,
    variable_names: Vec<String>,
}

impl<S: Solver> BuilderSolution<S> {
    pub(crate) fn new(solution: S::Solution, variable_names: Vec<String>) -> Self {
        Self {
            solution,
            variable_names,
        }
    }

    /// Returns the objective value of the solution.
    pub fn value(&self) -> f64 {
        self.solution.objective_value()
    }

    /// Returns the solved value of the variable referenced by `var`, or `None`
    /// if the handle does not belong to this model.
    pub fn var_value(&self, var: Var) -> Option<<S::Solution as Solution>::Value> {
        let name = self.variable_names.get(var.index)?;
        self.solution.var_value(name)
    }

    /// Returns the value of a variable as an `f64`.
    pub fn numeric_value(&self, var: Var) -> Option<f64> {
        self.var_value(var).map(Into::into)
    }

    /// Evaluates an arbitrary expression at the solution, resolving each
    /// variable to its solved value. Useful for reporting derived quantities.
    pub fn eval(&self, expr: &Expr) -> f64 {
        eval_expr(expr, &|idx| {
            self.variable_names
                .get(idx)
                .and_then(|name| self.solution.var_value(name))
                .map(Into::into)
                .unwrap_or(0.0)
        })
    }

    /// Returns the solver's underlying solution.
    pub fn solution(&self) -> &S::Solution {
        &self.solution
    }
}

impl<S: Solver> BuilderSolution<S>
where
    S::Solution: SolveStatus,
{
    /// Returns the solve status reported by the solver.
    pub fn status(&self) -> SolutionStatus {
        self.solution.status()
    }

    /// Returns why the search that produced this solution stopped.
    pub fn termination_reason(&self) -> TerminationReason {
        self.solution.termination_reason()
    }

    /// Returns the best objective bound proven by the search, when the solver
    /// reports one.
    pub fn best_bound(&self) -> Option<f64> {
        self.solution.best_bound()
    }

    /// Returns the relative gap between this solution and the best bound, when
    /// the solver reports one.
    pub fn gap(&self) -> Option<f64> {
        self.solution.gap()
    }
}

impl<S: Solver> BuilderSolution<S>
where
    S::Solution: ConstraintValues,
{
    /// Returns the activity (row value) of a named constraint.
    pub fn constraint_value(&self, constraint: &str) -> Option<f64> {
        self.solution.constraint_value(constraint)
    }
}

impl<S: Solver> BuilderSolution<S>
where
    S::Solution: DualValues,
{
    /// Returns the shadow price (dual value) of a named constraint.
    ///
    /// The value is the rate of change of the objective with respect to the
    /// constraint's right-hand side: raising the RHS of a binding `x >= 2` in a
    /// `min x` model by one raises the objective by one, so its shadow price is `+1`.
    /// Every backend reports it under that one convention - a sign that depended on
    /// which solver you picked would be unusable for sensitivity analysis.
    pub fn shadow_price(&self, constraint: &str) -> Option<f64> {
        self.solution.shadow_price(constraint)
    }
}

impl<S: Solver> BuilderSolution<S>
where
    S::Solution: ReducedCosts,
{
    /// Returns the reduced cost of a variable by name.
    pub fn reduced_cost(&self, variable: &str) -> Option<f64> {
        self.solution.reduced_cost(variable)
    }
}
