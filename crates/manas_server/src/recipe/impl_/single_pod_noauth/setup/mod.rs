//! I define traits and implementations for setup of [`SinglePodNoAuthRecipe``](super::SinglePodNoAuthRecipe`).
//!

use std::fmt::Debug;

use manas_repo_opendal::object_store::backend::BuildableODRObjectStoreBackend;

pub mod impl_;

/// A trait for setup of [`SinglePodNoAuthRecipe`](super::SinglePodNoAuthRecipe`).
pub trait SinglePodNoAuthRecipeSetup: Debug + Send + Sync + 'static {
    /// Recipe backend name in lowercase.
    const BACKEND_NAME: &'static str;

    /// Type of the backend builder.
    type BackendBuilder: opendal::Builder;

    /// Type of the backend.
    type Backend: BuildableODRObjectStoreBackend<Self::BackendBuilder>;
}
