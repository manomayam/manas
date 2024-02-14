//! I provide few implementations of [`Recipe`](super::Recipe).
//!

pub mod common;

pub mod single_pod_noauth;

#[cfg(feature = "layer-authentication")]
pub mod single_pod;
