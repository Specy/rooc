#[cfg(feature = "microlp")]
mod auto_solver;
#[cfg(feature = "clarabel")]
pub(crate) mod clarabel;
#[cfg(feature = "coin_cbc")]
pub(crate) mod coin_cbc;
pub mod common;
#[cfg(feature = "cplex-rs")]
pub(crate) mod cplex;
#[cfg(any(
    feature = "clarabel",
    feature = "coin_cbc",
    feature = "highs",
    feature = "lpsolve",
    feature = "scip",
    feature = "scip_bundled",
    feature = "lp-solvers",
    feature = "cplex-rs"
))]
pub(crate) mod good_lp;
#[cfg(feature = "highs")]
pub(crate) mod highs;
#[cfg(feature = "lp-solvers")]
pub(crate) mod lp_solvers;
#[cfg(feature = "lpsolve")]
pub(crate) mod lpsolve;
#[cfg(feature = "microlp")]
mod milp_solver;
#[cfg(any(feature = "scip", feature = "scip_bundled"))]
pub(crate) mod scip;
pub mod simplex;

#[cfg(feature = "clarabel")]
pub mod real_solver {
    pub use super::clarabel::solve_real_lp_problem_clarabel;
}

#[cfg(feature = "microlp")]
pub use auto_solver::*;
#[cfg(feature = "clarabel")]
pub use clarabel::solve_real_lp_problem_clarabel;
#[cfg(feature = "coin_cbc")]
pub use coin_cbc::solve_lp_problem_coin_cbc;
pub use common::*;
#[cfg(feature = "cplex-rs")]
pub use cplex::solve_lp_problem_cplex;
#[cfg(feature = "highs")]
pub use highs::solve_lp_problem_highs;
#[cfg(feature = "lp-solvers")]
pub use lp_solvers::solve_lp_problem_lp_solvers;
#[cfg(feature = "lpsolve")]
pub use lpsolve::solve_lp_problem_lpsolve;
#[cfg(feature = "microlp")]
pub use milp_solver::*;
#[cfg(any(feature = "scip", feature = "scip_bundled"))]
pub use scip::solve_lp_problem_scip;
pub use simplex::*;
