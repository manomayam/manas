//! I define few implementations of resource operator services.

mod delegating;
mod unsupported;

pub use delegating::*;
pub use unsupported::*;
