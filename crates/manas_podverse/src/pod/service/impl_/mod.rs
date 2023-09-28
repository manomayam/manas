//! I define few implementations of [`PodService`](super::PodService).
//!

mod basic;
mod overriden;
mod storage_describing;

pub use basic::*;
pub use overriden::*;
pub use storage_describing::*;
