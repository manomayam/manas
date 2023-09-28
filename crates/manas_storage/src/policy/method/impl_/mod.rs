//! I define few implementations of [`MethodPolicy`](super::MethodPolicy).
//!

/// I define a basic [`MethodPolicy`] that supports
/// patch method over rdf documents, along with other defaults.
///
mod rdf_patching;

pub use rdf_patching::*;
