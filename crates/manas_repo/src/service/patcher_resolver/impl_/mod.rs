//! I provide few implementations of [`RepPatcherResolver`](super::RepPatcherResolver).
//!

mod delegated;
mod unsupported;

pub use delegated::*;
pub use unsupported::*;
