//! An implementation of [access control policy](https://solid.github.io/authorization-panel/acp-specification/) concepts and engine for rust.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

pub mod model;

#[cfg(feature = "engine")]
pub mod attribute_match_svc;
#[cfg(feature = "engine")]
pub mod engine;
