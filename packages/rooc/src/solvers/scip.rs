//! SCIP backend provided by `good_lp`.

use super::good_lp::{GoodLpOptions, apply_mip_options, solve_with_good_lp};
use super::{LpSolution, SolverError};
use crate::transformers::LinearModel;
use indexmap::IndexMap;

/// Solves a mixed-integer linear model with SCIP.
pub fn solve_lp_problem_scip(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    solve_lp_problem_scip_with_options(lp, &GoodLpOptions::default())
}

pub(crate) fn solve_lp_problem_scip_with_options(
    lp: &LinearModel,
    options: &GoodLpOptions,
) -> Result<LpSolution<f64>, SolverError> {
    solve_with_good_lp(
        lp,
        ::good_lp::scip,
        |model, variables| apply_mip_options(model, options, variables),
        |_| Ok(()),
        |_, _| IndexMap::new(),
    )
}
