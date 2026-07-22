//! The IBM CPLEX mixed-integer solver.

use std::time::Duration;

use super::traits::Solver;
use crate::solvers::cplex::{solve_lp_problem_cplex, solve_lp_problem_cplex_with_options};
use crate::solvers::good_lp::GoodLpOptions;
use crate::solvers::{LpSolution, SolverError};
use crate::transformers::linear_model::LinearModel;

/// The IBM CPLEX mixed-integer solver.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cplex;

/// Configurable options for [`Cplex`].
#[derive(Debug, Clone, Default)]
pub struct CplexOptions {
    options: GoodLpOptions,
}

impl Cplex {
    /// Creates a configured solver with backend defaults.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CplexOptions {
        CplexOptions::default()
    }
}

impl CplexOptions {
    /// Sets a wall-clock limit for the solve.
    pub fn with_time_limit(mut self, limit: Duration) -> Self {
        self.options = self.options.with_time_limit(limit);
        self
    }
}

impl Solver for Cplex {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_cplex(model)
    }
}

impl Solver for CplexOptions {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_cplex_with_options(model, &self.options)
    }
}
