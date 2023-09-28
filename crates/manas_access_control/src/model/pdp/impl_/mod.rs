//! I define few implementations of [`PolicyDecisionPoint](super::PolicyDecisionPoint).
//!

#[cfg(feature = "impl-pdp-acp")]
pub mod acp;

#[cfg(feature = "impl-pdp-wac")]
pub mod wac;
