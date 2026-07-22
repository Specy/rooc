//! The Coin CBC mixed-integer solver.

use std::time::Duration;

use super::traits::Solver;
use crate::solvers::coin_cbc::{solve_lp_problem_coin_cbc, solve_lp_problem_coin_cbc_with_options};
use crate::solvers::good_lp::GoodLpOptions;
use crate::solvers::{LpSolution, SolverError};
use crate::transformers::linear_model::LinearModel;

/// The Coin CBC mixed-integer solver.
#[derive(Debug, Clone, Copy, Default)]
pub struct CoinCbc;

/// Configurable options for [`CoinCbc`].
#[derive(Debug, Clone, Default)]
pub struct CoinCbcOptions {
    options: GoodLpOptions,
}

impl CoinCbc {
    /// Creates a configured solver with backend defaults.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CoinCbcOptions {
        CoinCbcOptions::default()
    }
}

impl CoinCbcOptions {
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

impl Solver for CoinCbc {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_coin_cbc(model)
    }
}

impl Solver for CoinCbcOptions {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_coin_cbc_with_options(model, &self.options)
    }
}
