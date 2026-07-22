//! The SCIP mixed-integer solver.

use std::time::Duration;

use super::traits::Solver;
use crate::solvers::good_lp::GoodLpOptions;
use crate::solvers::scip::{solve_lp_problem_scip, solve_lp_problem_scip_with_options};
use crate::solvers::{LpSolution, SolverError};
use crate::transformers::linear_model::LinearModel;

/// The SCIP mixed-integer solver.
#[derive(Debug, Clone, Copy, Default)]
pub struct Scip;

/// Configurable options for [`Scip`].
#[derive(Debug, Clone, Default)]
pub struct ScipOptions {
    options: GoodLpOptions,
}

impl Scip {
    /// Creates a configured solver with backend defaults.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ScipOptions {
        ScipOptions::default()
    }
}

impl ScipOptions {
    /// Sets a wall-clock limit for the solve.
    pub fn with_time_limit(mut self, limit: Duration) -> Self {
        self.options = self.options.with_time_limit(limit);
        self
    }

    /// Sets the relative MIP gap at which the backend may stop early.
    /// Invalid values are reported as `SolverError::Other` when solving.
    pub fn with_mip_gap(mut self, gap: f64) -> Self {
        self.options = self.options.with_mip_gap(gap);
        self
    }

    /// Supplies a partial initial solution keyed by ROOC variable name.
    pub fn with_initial_solution<I, N>(mut self, solution: I) -> Self
    where
        I: IntoIterator<Item = (N, f64)>,
        N: Into<String>,
    {
        self.options = self.options.with_initial_solution(solution);
        self
    }
}

impl Solver for Scip {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_scip(model)
    }
}

impl Solver for ScipOptions {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_scip_with_options(model, &self.options)
    }
}
