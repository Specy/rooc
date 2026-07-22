//! lp_solve backend provided by `good_lp`.

use super::good_lp::{GoodLpOptions, apply_time_limit, solve_with_good_lp};
use super::{LpSolution, SolverError};
use crate::transformers::LinearModel;
use indexmap::IndexMap;

/// Solves a mixed-integer linear model with lp_solve.
pub fn solve_lp_problem_lpsolve(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    solve_lp_problem_lpsolve_with_options(lp, &GoodLpOptions::default())
}

pub(crate) fn solve_lp_problem_lpsolve_with_options(
    lp: &LinearModel,
    options: &GoodLpOptions,
) -> Result<LpSolution<f64>, SolverError> {
    solve_with_good_lp(
        lp,
        ::good_lp::lp_solve,
        |model, _| Ok(apply_time_limit(model, options)),
        |_| Ok(()),
        |_, _| IndexMap::new(),
    )
}
