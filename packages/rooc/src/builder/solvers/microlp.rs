//! The MicroLP mixed-integer solver.

use super::traits::Solver;
use crate::solvers::{LpSolution, MILPValue, MilpOptions, SolverError, solve_milp_lp_problem_with};
use crate::transformers::linear_model::LinearModel;
use std::time::Duration;

/// The MicroLP mixed-integer solver, with optional MIP gap and time limit.
#[derive(Debug, Clone, Default)]
pub struct Microlp {
    mip_gap: Option<f64>,
    time_limit: Option<Duration>,
}

impl Microlp {
    /// Creates a solver with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the relative MIP gap at which the search may stop early.
    pub fn with_mip_gap(mut self, gap: f64) -> Self {
        self.mip_gap = Some(gap);
        self
    }

    /// Sets a wall-clock limit for the search.
    pub fn with_time_limit(mut self, limit: Duration) -> Self {
        self.time_limit = Some(limit);
        self
    }
}

impl Solver for Microlp {
    type Solution = LpSolution<MILPValue>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        let options = MilpOptions {
            mip_gap: self.mip_gap,
            time_limit: self.time_limit,
        };
        solve_milp_lp_problem_with(model, &options)
    }
}
