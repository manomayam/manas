//! This crate Defines traits for access control systems compatible with solid storage space.
//! Provides default implementations confirming to [`ACP`](https://solid.github.io/authorization-panel/acp-specification/),
//! [`WAC`](https://solid.github.io/web-access-control-spec/) authorization systems.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

pub mod model;

#[cfg(feature = "impl-layered-repo")]
pub mod layered_repo;
