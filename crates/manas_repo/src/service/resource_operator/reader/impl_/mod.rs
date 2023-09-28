//! I define few implementations of [`ResourceReader`](super::ResourceReader).
//!

mod delegating;
mod unsupported;

pub use delegating::*;
pub use unsupported::*;
