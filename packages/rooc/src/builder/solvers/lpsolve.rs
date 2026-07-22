//! The lp_solve mixed-integer solver.

use std::time::Duration;

use super::traits::Solver;
use crate::solvers::good_lp::GoodLpOptions;
use crate::solvers::lpsolve::{solve_lp_problem_lpsolve, solve_lp_problem_lpsolve_with_options};
use crate::solvers::{LpSolution, SolverError};
use crate::transformers::linear_model::LinearModel;

/// The lp_solve mixed-integer solver.
#[derive(Debug, Clone, Copy, Default)]
pub struct LpSolve;

/// Configurable options for [`LpSolve`].
#[derive(Debug, Clone, Default)]
pub struct LpSolveOptions {
    options: GoodLpOptions,
}

impl LpSolve {
    /// Creates a configured solver with backend defaults.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> LpSolveOptions {
        LpSolveOptions::default()
    }
}

impl LpSolveOptions {
    /// Sets a wall-clock limit for the solve.
    pub fn with_time_limit(mut self, limit: Duration) -> Self {
        self.options = self.options.with_time_limit(limit);
        self
    }
}

impl Solver for LpSolve {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_lpsolve(model)
    }
}

impl Solver for LpSolveOptions {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_lpsolve_with_options(model, &self.options)
    }
}
