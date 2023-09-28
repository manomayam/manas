//! I define few implementations of [`NameLocker`](super::NameLocker).
//!

mod void;
pub use void::VoidNameLocker;

#[cfg(feature = "inmem")]
mod inmem;
#[cfg(feature = "inmem")]
pub use inmem::InmemNameLocker;
