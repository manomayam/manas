//! This crate provides implementations for few common repo
//! layers that integrate into manas eco system.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

#[cfg(feature = "dconneging")]
pub mod dconneging;

pub mod delegating;

#[cfg(feature = "patching")]
pub mod patching;

#[cfg(feature = "validating")]
pub mod validating;
