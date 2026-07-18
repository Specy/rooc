//! The builder's solver layer: shared traits and the built-in solvers.

mod auto;
mod clarabel;
mod microlp;
mod traits;

pub use auto::*;
pub use clarabel::*;
pub use microlp::*;
pub use traits::*;
