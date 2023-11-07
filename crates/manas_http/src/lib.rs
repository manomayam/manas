//! This crate provides extended functionality for handling http semantics, that integrates into [`hyper`](https://docs.rs/hyper/latest/hyper/index.html) ecosystem.
//!
//! See [RFC 9110 HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html)
//!

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unused_qualifications)]

#[cfg(feature = "typed-headers")]
pub mod field;

#[cfg(feature = "typed-headers")]
pub mod header;

#[cfg(feature = "conditional_req")]
pub mod conditional_req;

#[cfg(feature = "representation")]
pub mod representation;

#[cfg(feature = "service")]
pub mod service;

pub mod uri;
