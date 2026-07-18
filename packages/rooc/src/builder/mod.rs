//! Fluent builder API: build optimization models in code and solve them.

mod expr;
mod macros;
mod model;
mod solution;
pub mod solvers;

pub use expr::{Expr, Var, abs, all, any, max, min, sum};
pub use model::{BuilderConstraint, BuilderError, ModelBuilder};
pub use solution::BuilderSolution;
pub use solvers::{
    Auto, Clarabel, ConstraintValues, DualValues, Microlp, ReducedCosts, Solution, SolveStatus,
    Solver,
};
