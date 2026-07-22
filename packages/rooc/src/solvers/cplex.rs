//! IBM CPLEX backend provided by `good_lp`.

use super::good_lp::{GoodLpOptions, apply_time_limit, solve_with_good_lp};
use super::{LpSolution, SolverError};
use crate::transformers::LinearModel;
use indexmap::IndexMap;

/// Solves a mixed-integer linear model with IBM CPLEX.
pub fn solve_lp_problem_cplex(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    solve_lp_problem_cplex_with_options(lp, &GoodLpOptions::default())
}

pub(crate) fn solve_lp_problem_cplex_with_options(
    lp: &LinearModel,
    options: &GoodLpOptions,
) -> Result<LpSolution<f64>, SolverError> {
    solve_with_good_lp(
        lp,
        ::good_lp::solvers::cplex::cplex,
        |model, _| Ok(apply_time_limit(model, options)),
        |_| Ok(()),
        |_, _| IndexMap::new(),
    )
}
