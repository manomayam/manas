//! I define few implementations of [`ResourceDeleter`](super::ResourceDeleter).
//!

mod delegating;
mod unsupported;

pub use delegating::*;
pub use unsupported::*;
