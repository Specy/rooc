//! HiGHS backend provided by `good_lp`.

use super::good_lp::{GoodLpOptions, apply_mip_options, collect_good_lp_duals, solve_with_good_lp};
use super::{LpSolution, SolverError};
use crate::transformers::LinearModel;
use ::good_lp::SolutionWithDual;

/// Solves a mixed-integer linear model with HiGHS.
pub fn solve_lp_problem_highs(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    solve_lp_problem_highs_with_options(lp, &GoodLpOptions::default())
}

pub(crate) fn solve_lp_problem_highs_with_options(
    lp: &LinearModel,
    options: &GoodLpOptions,
) -> Result<LpSolution<f64>, SolverError> {
    solve_with_good_lp(
        lp,
        ::good_lp::highs,
        |model, variables| apply_mip_options(model, options, variables),
        |_| Ok(()),
        |solution, references| {
            let dual = solution.compute_dual();
            collect_good_lp_duals(dual, references)
        },
    )
}
