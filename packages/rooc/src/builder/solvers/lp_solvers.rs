//! External command-line solvers selected through `lp-solvers`.

use super::traits::Solver;
use crate::solvers::{LpSolution, SolverError, solve_lp_problem_lp_solvers};
use crate::transformers::linear_model::LinearModel;

/// An external solver selected through `lp-solvers`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LpSolvers;

impl Solver for LpSolvers {
    type Solution = LpSolution<f64>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        solve_lp_problem_lp_solvers(model)
    }
}
