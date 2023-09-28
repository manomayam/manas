//! This crate provides utilities to deal with rdf data.
//!

#![warn(missing_docs)]

pub mod model;

#[cfg(feature = "query")]
pub mod query;

pub mod patch;
