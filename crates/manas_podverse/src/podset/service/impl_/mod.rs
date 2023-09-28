//! I define few implementations of [`PodSetservice`](super::PodSetService).
//!

mod basic;
mod overriden;

pub use basic::BasicPodSetService;
pub use overriden::OverridenPodSetService;
