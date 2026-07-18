//! The builder's solution wrapper and capability forwarding.

use super::expr::{Expr, Var, eval_expr};
use super::model::{BuilderConstraint, BuilderError, ModelBuilder};
use super::solvers::{
    ConstraintValues, DualValues, ReducedCosts, Reoptimizable, Reoptimize, SolveStatus, Solution,
    Solver,
};
use crate::solvers::SolutionStatus;

/// A solution produced by [`ModelBuilder::solve_with`].
///
/// Wraps the solver's own [`Solution`] and lets you read values back through the
/// [`Var`] handles from [`ModelBuilder::add_var`]. Optional capabilities
/// (`status`, `constraint_value`, `shadow_price`, `reduced_cost`) are available
/// only when the solver's solution implements the matching trait; editing and
/// re-solving ([`Reoptimize`]) is available when the solver is [`Reoptimizable`].
pub struct BuilderSolution<S: Solver> {
    solution: S::Solution,
    variable_names: Vec<String>,
    model: ModelBuilder,
    solver: S,
}

impl<S: Solver> BuilderSolution<S> {
    pub(crate) fn new(
        solution: S::Solution,
        variable_names: Vec<String>,
        model: ModelBuilder,
        solver: S,
    ) -> Self {
        Self {
            solution,
            variable_names,
            model,
            solver,
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

impl<S: Solver + Reoptimizable> Reoptimize for BuilderSolution<S> {
    fn with(mut self, constraint: BuilderConstraint) -> Self {
        self.model = self.model.with(constraint);
        self
    }

    fn with_all(mut self, constraints: impl IntoIterator<Item = BuilderConstraint>) -> Self {
        self.model = self.model.with_all(constraints);
        self
    }

    fn resolve(self) -> Result<Self, BuilderError> {
        self.model.solve_with(self.solver)
    }
}
