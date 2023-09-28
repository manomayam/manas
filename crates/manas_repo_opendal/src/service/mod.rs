//! This module implements repo services for ODR.
//!

/// I provide implementations of resource operators for odr.
pub mod resource_operator;

/// I define initializer for ODR.
pub mod initializer;

#[cfg(feature = "access-prp")]
/// I define access control policy retrieval point for ODR.
pub mod prp;
