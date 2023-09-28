//! I define few implementations of [`ResourceCreator`](super::ResourceCreator).
//!

mod delegating;
mod unsupported;

pub use delegating::*;
pub use unsupported::*;
