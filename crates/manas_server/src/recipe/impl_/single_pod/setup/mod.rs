//! I define traits and implementations for setup of [`SinglePodRecipe``](super::SinglePodRecipe`).
//!

use std::{collections::HashSet, fmt::Debug};

use manas_access_control::model::pdp::PolicyDecisionPoint;
use manas_repo_opendal::object_store::backend::BuildableODRObjectStoreBackend;
use rdf_utils::model::triple::ArcTriple;

use crate::space::RcpStorageSpace;

pub mod impl_;

/// A trait for setup of [`SinglePodRecipe`](super::SinglePodRecipe`).
pub trait SinglePodRecipeSetup: Debug + Send + Sync + 'static {
    /// Recipe backend name in lowercase.
    const BACKEND_NAME: &'static str;

    /// Recipe pdp name in lowercase.
    const PDP_NAME: &'static str;

    /// Initial root acr template str.
    const INITIAL_ROOT_ACR_TEMPLATE: &'static str;

    /// Type of the pdp.
    type PDP: PolicyDecisionPoint<StSpace = RcpStorageSpace, Graph = HashSet<ArcTriple>>
        +
        // For now.
        Default;

    /// Type of the backend builder.
    type BackendBuilder: opendal::Builder;

    /// Type of the backend.
    type Backend: BuildableODRObjectStoreBackend<Self::BackendBuilder>;
}
