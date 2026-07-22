//! The builder's solver layer: shared traits and the built-in solvers.

#[cfg(feature = "microlp")]
mod auto;
#[cfg(feature = "clarabel")]
mod clarabel;
#[cfg(feature = "coin_cbc")]
mod coin_cbc;
#[cfg(feature = "cplex-rs")]
mod cplex;
#[cfg(feature = "highs")]
mod highs;
#[cfg(feature = "lp-solvers")]
mod lp_solvers;
#[cfg(feature = "lpsolve")]
mod lpsolve;
#[cfg(feature = "microlp")]
mod microlp;
#[cfg(any(feature = "scip", feature = "scip_bundled"))]
mod scip;
mod traits;

#[cfg(feature = "microlp")]
pub use auto::*;
#[cfg(feature = "clarabel")]
pub use clarabel::*;
#[cfg(feature = "coin_cbc")]
pub use coin_cbc::*;
#[cfg(feature = "cplex-rs")]
pub use cplex::*;
#[cfg(feature = "highs")]
pub use highs::*;
#[cfg(feature = "lp-solvers")]
pub use lp_solvers::*;
#[cfg(feature = "lpsolve")]
pub use lpsolve::*;
#[cfg(feature = "microlp")]
pub use microlp::*;
#[cfg(any(feature = "scip", feature = "scip_bundled"))]
pub use scip::*;
pub use traits::*;
