//! This crate provides utilities to deal with rdf data.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]

pub mod model;

#[cfg(feature = "query")]
pub mod query;

pub mod patch;
