//! I define few implementations of [`PolicyRetrievalPoint`](super::PolicyRetrievalPoint).
//!

#[cfg(feature = "cache-layered-prp")]
mod cache_layered;

#[cfg(feature = "cache-layered-prp")]
pub use cache_layered::*;
