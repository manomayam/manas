//! This crate provides extended functionality for handling http semantics, that integrates into [`hyper`](https://docs.rs/hyper/latest/hyper/index.html) ecosystem.
//!
//! See [RFC 9110 HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html)
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

#[cfg(feature = "typed-headers")]
pub mod header;

#[cfg(feature = "conditional_req")]
pub mod conditional_req;

#[cfg(feature = "representation")]
pub mod representation;

#[cfg(feature = "service")]
pub mod service;

pub mod uri;

#[cfg(feature = "typed-headers")]
pub use http_headers::define_static_rel_types;
