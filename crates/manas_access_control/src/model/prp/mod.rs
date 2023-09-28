//! I define interfaces and implementations for
//! access control policy-retrieval-points.
//!

use std::{borrow::Borrow, fmt::Debug};

use acp::model::acr::DAccessControlResource;
use dyn_problem::{ProbFuture, ProbStream};
use http_uri::invariant::NormalAbsoluteHttpUri;
use manas_space::{resource::slot::SolidResourceSlot, SolidStorageSpace};
use rdf_utils::model::graph::InfallibleGraph;

pub mod impl_;

/// A trait for access control policy retrieval points.
pub trait PolicyRetrievalPoint: Debug + Send + Sync + 'static {
    /// Type of the solid storage space.
    type StSpace: SolidStorageSpace;

    /// Type of graphs.
    // TODO add  + SetGraph bound.
    type Graph: InfallibleGraph + Send + Sync + 'static;

    /// Type of wrapped graphs.
    type WGraph: Borrow<Self::Graph>;

    /// Retrieve the acr chain for given resource uri.
    fn retrieve(
        &self,
        resource_uri: NormalAbsoluteHttpUri,
        deduced_containment_is_sufficient: bool,
    ) -> ProbFuture<'static, SlotAcrChain<Self::StSpace, Self::Graph, Self::WGraph>>;
}

/// A type alias for slot acr chain stream.
pub type SlotAcrChain<StSpace, G, WG> = ProbStream<'static, SlotAcrChainItem<StSpace, G, WG>>;

/// A struct to represent an item in slot acr chain.
#[derive(Debug, Clone)]
pub struct SlotAcrChainItem<StSpace, G, WG>
where
    StSpace: SolidStorageSpace,
    // TODO add SetGraph bound.
    G: InfallibleGraph + Send + 'static,
    WG: Borrow<G>,
{
    /// Slot of the resource.
    pub res_slot: SolidResourceSlot<StSpace>,

    /// Acr associated with the resource.
    // TODO Should wrap in `Representation` instead?
    pub acr: Option<DAccessControlResource<G, WG>>,
}
