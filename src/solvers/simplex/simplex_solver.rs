use crate::solvers::{LpSolution, SimplexError, SolverError};
use crate::transformers::LinearModel;


#[allow(unused)]
pub fn solve_lp_problem(lp: &LinearModel, limit: i64) -> Result<LpSolution<f64>, SolverError> {
    let standard = lp.clone().into_standard_form()?;
    let mut canonical_form = standard
        .into_canonical()
        .map_err(|e| SolverError::Other(e.to_string()))?;

    let solution = canonical_form.solve(limit);
    match solution {
        Ok(optimal_tableau) => Ok(optimal_tableau.as_lp_solution()),
        Err(e) => match e {
            SimplexError::IterationLimitReached => Err(SolverError::LimitReached),
            SimplexError::Unbounded => Err(SolverError::Unbounded),
            SimplexError::Other => Err(SolverError::Other("An error occoured".to_string())),
        },
    }
}
