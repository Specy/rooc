//! The safe general-purpose MILP solver selection.

use super::traits::Solver;
use crate::solvers::{LpSolution, MILPValue, SolverError, auto_solver};
use crate::transformers::linear_model::LinearModel;

/// Uses the [`Microlp`](super::Microlp) MILP backend for every supported model.
/// Choose [`Clarabel`](super::Clarabel) explicitly for a continuous LP.
#[derive(Debug, Clone, Copy, Default)]
pub struct Auto;

impl Solver for Auto {
    type Solution = LpSolution<MILPValue>;

    fn solve(&self, model: &LinearModel) -> Result<Self::Solution, SolverError> {
        auto_solver(model)
    }
}
