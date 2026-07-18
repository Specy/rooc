//! The Clarabel continuous-LP solver.

use super::traits::Solver;
use crate::solvers::{LpSolution, SolverError, solve_real_lp_problem_clarabel};
use crate::transformers::linear_model::LinearModel;

/// The Clarabel solver for continuous (real) linear programs.
#[derive(Debug, Clone, Copy, Default)]
pub struct Clarabel;

impl Solver for Clarabel {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_real_lp_problem_clarabel(model)
    }
}
