//! This crate provides few typed http headers.
//!
//! ## Feature flags
#![cfg_attr(
    doc_cfg,
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

pub mod common;

#[cfg(feature = "accept")]
pub mod accept;

#[cfg(feature = "accept-method")]
pub mod accept_patch;
#[cfg(feature = "accept-method")]
pub mod accept_post;
#[cfg(feature = "accept-method")]
pub mod accept_put;

#[cfg(feature = "link")]
pub mod link;

#[cfg(feature = "location")]
pub mod location;

#[cfg(feature = "prefer")]
pub mod prefer;
#[cfg(feature = "prefer")]
pub mod preference_applied;

#[cfg(feature = "slug")]
pub mod slug;

#[cfg(feature = "forwarded")]
pub mod forwarded;
#[cfg(feature = "forwarded")]
pub mod x_forwarded_host;
#[cfg(feature = "forwarded")]
pub mod x_forwarded_proto;

#[cfg(feature = "wac-allow")]
pub mod wac_allow;

#[cfg(feature = "www-authenticate")]
pub mod www_authenticate;
