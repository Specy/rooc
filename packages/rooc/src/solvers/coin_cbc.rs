//! Coin CBC backend provided by `good_lp`.

use super::good_lp::{GoodLpOptions, apply_mip_options, solve_with_good_lp};
use super::{LpSolution, SolveOutcome, SolverError};
use crate::math::VariableType;
use crate::transformers::LinearModel;
use indexmap::IndexMap;

/// Solves a mixed-integer linear model with Coin CBC.
pub fn solve_lp_problem_coin_cbc(
    lp: &LinearModel,
) -> Result<SolveOutcome<LpSolution<f64>>, SolverError> {
    solve_lp_problem_coin_cbc_with_options(lp, &GoodLpOptions::default())
}

pub(crate) fn solve_lp_problem_coin_cbc_with_options(
    lp: &LinearModel,
    options: &GoodLpOptions,
) -> Result<SolveOutcome<LpSolution<f64>>, SolverError> {
    // A best bound is what branch and bound proves along the way, so it is only
    // read back for models that actually have something to branch on. A purely
    // continuous model is left without one rather than reporting whatever CBC
    // happens to hold when it never opened a search tree.
    let is_discrete = lp.domain().values().any(|variable| {
        matches!(
            variable.get_type(),
            VariableType::Boolean | VariableType::IntegerRange(_, _)
        )
    });

    solve_with_good_lp(
        lp,
        ::good_lp::coin_cbc,
        |model, variables| apply_mip_options(model, options, variables),
        |_| Ok(()),
        |_, _| IndexMap::new(),
        |solution| {
            if !is_discrete {
                return None;
            }
            // `good_lp` sets objective coefficients on CBC but never forwards the
            // objective's constant term, so the bound is measured on the un-offset
            // objective and the offset is added back here to keep it comparable
            // with the reported objective value. An infinite bound means CBC never
            // proved one, so it is reported as absent rather than as a real bound.
            let bound = solution.model().best_possible_value();
            if bound.is_finite() {
                Some(bound + lp.objective_offset())
            } else {
                None
            }
        },
    )
}
