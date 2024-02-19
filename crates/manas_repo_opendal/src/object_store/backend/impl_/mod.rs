//! I provide few implementations of the [`ODRObjectStoreBackend`](super::ODRObjectStoreBackend).
//!

pub mod common;

#[cfg(feature = "backend-fs")]
pub mod fs;

#[cfg(feature = "backend-s3")]
pub mod s3;

#[cfg(feature = "backend-gcs")]
pub mod gcs;

#[cfg(feature = "backend-embedded")]
pub mod embedded;

#[cfg(feature = "backend-inmem")]
pub mod inmem;
