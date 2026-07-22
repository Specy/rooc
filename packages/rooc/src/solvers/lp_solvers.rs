//! External command-line backends provided through `lp-solvers`.

use super::good_lp::solve_with_good_lp;
use super::{LpSolution, SolverError};
use crate::transformers::LinearModel;
use indexmap::IndexMap;

/// Solves a mixed-integer linear model through an external solver selected by `lp-solvers`.
pub fn solve_lp_problem_lp_solvers(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    let solver =
        ::good_lp::solvers::lp_solvers::LpSolver(::good_lp::solvers::lp_solvers::StaticSolver::<
            ::good_lp::solvers::lp_solvers::AllSolvers,
        >::new());
    solve_with_good_lp(
        lp,
        solver,
        |model, _| Ok(model),
        |_| Ok(()),
        |_, _| IndexMap::new(),
    )
}
