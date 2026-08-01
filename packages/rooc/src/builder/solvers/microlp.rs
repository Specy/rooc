//! The MicroLP mixed-integer solver.

use super::traits::Solver;
use crate::solvers::{
    LpSolution, MILPValue, MilpOptions, SolveOutcome, SolverError, solve_milp_lp_problem_with,
};
use crate::transformers::linear_model::LinearModel;
use std::time::Duration;

/// The MicroLP mixed-integer solver, with optional MIP gap, time limit, and
/// node limit.
#[derive(Debug, Clone, Default)]
pub struct Microlp {
    mip_gap: Option<f64>,
    time_limit: Option<Duration>,
    node_limit: Option<u64>,
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

    /// Sets the maximum number of branch-and-bound nodes to explore.
    ///
    /// A deterministic alternative to [`Microlp::with_time_limit`]. It has no
    /// effect on a model without integer or boolean variables, which is solved
    /// without branching.
    pub fn with_node_limit(mut self, limit: u64) -> Self {
        self.node_limit = Some(limit);
        self
    }
}

impl Solver for Microlp {
    type Solution = LpSolution<MILPValue>;

    fn solve(&self, model: &LinearModel) -> Result<SolveOutcome<Self::Solution>, SolverError> {
        let options = MilpOptions {
            mip_gap: self.mip_gap,
            time_limit: self.time_limit,
            node_limit: self.node_limit,
        };
        solve_milp_lp_problem_with(model, &options)
    }
}
