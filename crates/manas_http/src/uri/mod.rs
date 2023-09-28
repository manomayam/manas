//! Uniform Resource Identifiers (URIs) are used throughout HTTP as the means for identifying resources (Section 3.1).
//!

pub mod component;

pub use http_uri::{invariant, predicate, *};
