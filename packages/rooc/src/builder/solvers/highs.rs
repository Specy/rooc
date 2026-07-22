//! The HiGHS mixed-integer solver.

use std::time::Duration;

use super::traits::Solver;
use crate::solvers::good_lp::GoodLpOptions;
use crate::solvers::highs::{solve_lp_problem_highs, solve_lp_problem_highs_with_options};
use crate::solvers::{LpSolution, SolverError};
use crate::transformers::linear_model::LinearModel;

/// The HiGHS mixed-integer solver.
#[derive(Debug, Clone, Copy, Default)]
pub struct Highs;

/// Configurable options for [`Highs`].
#[derive(Debug, Clone, Default)]
pub struct HighsOptions {
    options: GoodLpOptions,
}

impl Highs {
    /// Creates a configured solver with backend defaults.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> HighsOptions {
        HighsOptions::default()
    }
}

impl HighsOptions {
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

impl Solver for Highs {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_highs(model)
    }
}

impl Solver for HighsOptions {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_highs_with_options(model, &self.options)
    }
}
