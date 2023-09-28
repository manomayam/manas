//! I define few implementations of [`ResourceUpdater`](super::ResourceUpdater).
//!

mod delegating;
mod unsupported;

pub use delegating::*;
pub use unsupported::*;
