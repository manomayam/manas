//! This crate provides utilities to deal with rdf data.
//!

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod model;

#[cfg(feature = "query")]
pub mod query;

pub mod patch;
