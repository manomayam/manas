//! This crate provides implementations of various http authentication schemes
//! for solid storage resource servers and authorization servers.

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

pub mod common;

#[cfg(feature = "cr-framework")]
pub mod challenge_response_framework;
