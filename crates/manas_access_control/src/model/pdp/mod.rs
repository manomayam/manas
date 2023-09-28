//! I define interfaces and implementations for access control
//! policy-decision-points.
//!

use std::{collections::HashSet, fmt::Debug, ops::Deref, sync::Arc};

use acp::model::{access_mode::HAccessMode, attribute::HAttribute, context::DContext};
use dyn_problem::{define_anon_problem_types, ProbFuture};
use http_uri::invariant::NormalAbsoluteHttpUri;
use itertools::Itertools;
use manas_space::{resource::slot::SolidResourceSlot, SolidStorageSpace};
use rdf_utils::model::{
    graph::{InfallibleGraph, InfallibleMutableGraph},
    handle::Handle,
    term::ArcTerm,
};
use sophia_api::term::Term;

use super::AccessGrantSet;
use crate::model::prp::SlotAcrChain;

pub mod impl_;

/// A struct to represent access grant response.
#[derive(Debug, Clone)]
pub struct AccessGrantResponse<Space: SolidStorageSpace> {
    /// Slot of the resource. [`None`] implies resource
    /// doesn't exist.
    pub res_slot: Option<SolidResourceSlot<Space>>,

    /// Resolved access grant set on the resource.
    pub access_grant_set: AccessGrantSet,
}

/// A trait for access control policy decision points.
/// A pdp is a centralized point which resolves decisions
/// regarding access to resources in a storage space.
///
pub trait PolicyDecisionPoint: Debug + Send + Sync + 'static {
    /// Type of solid storage space.
    type StSpace: SolidStorageSpace;

    /// Type of graphs.
    type Graph: InfallibleMutableGraph + Default + Send + Sync + 'static;

    /// Get the slice of access modes supported by this pdp.
    fn supported_access_modes(&self) -> &HashSet<HAccessMode<ArcTerm>>;

    /// Get the slice of attributes supported by this pdp.
    fn supported_attrs(&self) -> &HashSet<HAttribute<ArcTerm>>;

    /// Resolve the granted access modes for given access context.
    fn resolve_grants(
        &self,
        context: ResourceAccessContext<Self::Graph>,
        acr_chain: SlotAcrChain<Self::StSpace, Self::Graph, Arc<Self::Graph>>,
    ) -> ProbFuture<'static, AccessGrantResponse<Self::StSpace>>;
}

/// A struct to represent resource access context.
pub struct ResourceAccessContext<G: InfallibleGraph> {
    target_uri: NormalAbsoluteHttpUri,
    inner: DContext<G, G>,
}

impl<G: InfallibleGraph> TryFrom<DContext<G, G>> for ResourceAccessContext<G> {
    type Error = InvalidTargetAttribute;

    fn try_from(context: DContext<G, G>) -> Result<Self, Self::Error> {
        // Get handle to the target resource.
        let h_target = context
            .h_target::<ArcTerm>()
            .exactly_one()
            .map_err(|_| InvalidTargetAttribute::InvalidCardinality)?;

        // Get target resource uri.
        let target_uri = h_target
            .as_term()
            .iri()
            .and_then(|iri| NormalAbsoluteHttpUri::try_new_from(iri.as_str()).ok())
            .ok_or(InvalidTargetAttribute::UriIsNotNormalAbsolute)?;

        Ok(Self {
            target_uri,
            inner: context,
        })
    }
}

impl<G: InfallibleGraph> Deref for ResourceAccessContext<G> {
    type Target = DContext<G, G>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<G: InfallibleGraph> ResourceAccessContext<G> {
    /// Create a new access context with out checks.
    /// Caller must ensure that inner context has
    /// matching target uri.
    #[inline]
    pub(crate) fn new_unchecked(target_uri: NormalAbsoluteHttpUri, inner: DContext<G, G>) -> Self {
        Self { target_uri, inner }
    }

    /// Get access target uri.
    #[inline]
    pub fn target_uri(&self) -> &NormalAbsoluteHttpUri {
        &self.target_uri
    }

    /// Convert into inner access context.
    #[inline]
    pub fn into_inner(self) -> DContext<G, G> {
        self.inner
    }
}

/// Invalid  `acp:target`  attribute in access context graph.
#[derive(Debug, thiserror::Error)]
pub enum InvalidTargetAttribute {
    /// Invalid cardinality of `acp:target` in access context graph.
    #[error("Invalid cardinality of `acp:target` in access context graph.")]
    InvalidCardinality,

    /// acp:target attribute value is not a normal absolute http uri.
    #[error("acp:target attribute value is not a normal absolute http uri.")]
    UriIsNotNormalAbsolute,
}

define_anon_problem_types!(
    /// Unknown target resource.
    UNKNOWN_TARGET_RESOURCE: (
        "Unknown target resource."
    );

    /// Invalid prp response.
    INVALID_PRP_RESPONSE: (
        "Invalid prp response."
    );
);
