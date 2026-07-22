//! Fluent builder API: build optimization models in code and solve them.

mod expr;
mod macros;
mod model;
mod solution;
pub mod solvers;

pub use expr::{Expr, Var, abs, all, any, max, min, sum};
pub use model::{BuilderConstraint, BuilderError, ModelBuilder};
pub use solution::BuilderSolution;
#[cfg(feature = "clarabel")]
pub use solvers::Clarabel;
#[cfg(feature = "coin_cbc")]
pub use solvers::CoinCbc;
#[cfg(feature = "cplex-rs")]
pub use solvers::Cplex;
#[cfg(feature = "highs")]
pub use solvers::Highs;
#[cfg(feature = "lpsolve")]
pub use solvers::LpSolve;
#[cfg(feature = "lp-solvers")]
pub use solvers::LpSolvers;
#[cfg(any(feature = "scip", feature = "scip_bundled"))]
pub use solvers::Scip;
#[cfg(feature = "microlp")]
pub use solvers::{Auto, Microlp};
pub use solvers::{ConstraintValues, DualValues, ReducedCosts, Solution, SolveStatus, Solver};
