//! I define an implementation of [`PolicyDecisionPoint`]
//! that confirms to `WAC` specification.
//!

use std::{collections::HashSet, fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use acp::model::{
    access_mode::{HAccessMode, H_APPEND, H_CONTROL, H_READ, H_WRITE},
    attribute::HAttribute,
};
use async_recursion::async_recursion;
use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, ProbResult};
use futures::{StreamExt, TryFutureExt};
use http_uri::invariant::NormalAbsoluteHttpUri;
use manas_space::{
    resource::{
        slot_rel_type::{
            aux_rel_type::known::{AuxAccessResolutionRole, KnownAuxRelType},
            SlotRelationType,
        },
        slot_rev_link::SlotRevLink,
    },
    SolidStorageSpace,
};
use rdf_utils::model::{
    description::DescriptionExt, graph::InfallibleMutableGraph, handle::Handle, term::ArcTerm,
};
use rdf_vocabularies::ns;
use sophia_api::{graph::SetGraph, term::Term};
use tracing::{error, info};

use self::engine::{ResolvedAclContext, WacEngine};
use crate::model::{
    pdp::{AccessGrantResponse, PolicyDecisionPoint, ResourceAccessContext, INVALID_PRP_RESPONSE},
    prp::SlotAcrChain,
};

pub mod engine;

/// An implementation of [`PolicyDecisionPoint`] that
/// confirms to `WAC` specification.
#[derive(Clone)]
pub struct WacDecisionPoint<S, G>
where
    G: InfallibleMutableGraph + Default + Clone,
    S: SolidStorageSpace,
{
    /// Wac engine.
    engine: WacEngine<G, Arc<G>>,

    ///Supported access modes.
    supported_access_modes: Arc<HashSet<HAccessMode<ArcTerm>>>,

    ///Supported attributes.
    supported_attrs: Arc<HashSet<HAttribute<ArcTerm>>>,

    _phantom: PhantomData<fn(S)>,
}

impl<S, G> Debug for WacDecisionPoint<S, G>
where
    G: InfallibleMutableGraph + Default + Clone + Debug,
    S: SolidStorageSpace,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WacDecisionPoint").finish()
    }
}

impl<S, G> Default for WacDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + Default + Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        let engine = WacEngine::default();
        let supported_attrs = Arc::new(
            engine
                .supported_attrs()
                .filter_map(|a| HAttribute::try_new(a).ok())
                .collect(),
        );
        Self::new(
            engine,
            Arc::new(
                [&H_READ, &H_APPEND, &H_WRITE, &H_CONTROL]
                    .into_iter()
                    .map(|m| m.clone().map_term())
                    .collect(),
            ),
            supported_attrs,
        )
    }
}

impl<S, G> WacDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + Default + Clone,
{
    /// Create a new [`WacDecisionPoint`] with given params.
    #[inline]
    pub fn new(
        engine: WacEngine<G, Arc<G>>,
        supported_access_modes: Arc<HashSet<HAccessMode<ArcTerm>>>,
        supported_attrs: Arc<HashSet<HAttribute<ArcTerm>>>,
    ) -> Self {
        Self {
            engine,
            supported_access_modes,
            supported_attrs,
            _phantom: PhantomData,
        }
    }
}

impl<S, G> PolicyDecisionPoint for WacDecisionPoint<S, G>
where
    G: InfallibleMutableGraph + SetGraph + Debug + Default + Clone + Send + Sync + 'static,
    S: SolidStorageSpace,
{
    type StSpace = S;

    type Graph = G;

    #[inline]
    fn supported_access_modes(&self) -> &HashSet<HAccessMode<ArcTerm>> {
        &self.supported_access_modes
    }

    #[inline]
    fn supported_attrs(&self) -> &HashSet<HAttribute<ArcTerm>> {
        &self.supported_attrs
    }

    #[tracing::instrument(skip_all, name = "WacDecisionPoint::resolve_grants")]
    fn resolve_grants(
        &self,
        context: ResourceAccessContext<Self::Graph>,
        acr_chain: SlotAcrChain<Self::StSpace, Self::Graph, Arc<Self::Graph>>,
    ) -> ProbFuture<'static, AccessGrantResponse<Self::StSpace>> {
        let engine = self.engine.clone();
        let supported_access_modes = self.supported_access_modes.clone();

        Box::pin(async move {
            Self::resolve_grants_with(context, acr_chain, engine, supported_access_modes).await
        })
    }
}

impl<S, G> WacDecisionPoint<S, G>
where
    G: InfallibleMutableGraph + SetGraph + Debug + Default + Clone + Send + Sync + 'static,
    S: SolidStorageSpace,
{
    /// Resolve grants with given context and acr chain.
    #[async_recursion]
    async fn resolve_grants_with(
        context: ResourceAccessContext<G>,
        mut acl_chain: SlotAcrChain<S, G, Arc<G>>,
        engine: WacEngine<G, Arc<G>>,
        supported_access_modes: Arc<HashSet<HAccessMode<ArcTerm>>>,
    ) -> ProbResult<AccessGrantResponse<S>> {
        let target_uri = context.target_uri().clone();

        // Retrieve own item.
        let own_item = acl_chain
            .next()
            .await
            .ok_or_else(|| {
                error!(
                    "PRP doesn't provide acr chain item for {}",
                    target_uri.as_str()
                );
                INVALID_PRP_RESPONSE.new_problem()
            })?
            .map_err(|e| {
                error!(
                    "Unknown io error in retrieving acr for {}",
                    target_uri.as_str()
                );
                UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
            })?;

        // Ensure target consistency.
        if target_uri != own_item.res_slot.id().uri {
            error!(
                "Target uri mismatch with acr-chain-item resource uri. expected: {}, got: {}",
                target_uri.as_str(),
                own_item.res_slot.id().uri.as_str()
            );
            return Err(INVALID_PRP_RESPONSE.new_problem());
        }

        let access_grant_set = match &own_item.res_slot.slot_rev_link() {
            // If resource has no slot rev link, implies it is
            // the storage root.
            None => {
                info!(
                    "Storage root is resolved to be nearest ipr. IPR uri: {}",
                    target_uri.as_str()
                );

                engine
                    .resolve_access_control(
                        own_item.acr.map(|acl| ResolvedAclContext {
                            target_uri: target_uri.clone(),
                            resolved_acl_subject_uri: own_item.res_slot.id().uri.clone(),
                            resolved_acl: acl,
                        }),
                        context.into_inner().into_with_arced_graph(),
                    )
                    .inspect_err(|_| {
                        error!("Error in resolving grants for nearest ipr");
                    })
                    .await
            }

            Some(SlotRevLink {
                target: host_res_uri,
                rev_rel_type,
            }) => match rev_rel_type {
                // Resource itself is an ipr, as all contained
                // resources are iprs.
                SlotRelationType::Contains => {
                    info!("Resolved nearest ipr uri: {}", target_uri.as_str());

                    engine
                        .resolve_access_control(
                            if let Some(acl) = own_item.acr {
                                Some(ResolvedAclContext {
                                    target_uri: target_uri.clone(),
                                    resolved_acl_subject_uri: own_item.res_slot.id().uri.clone(),
                                    resolved_acl: acl,
                                })
                            } else {
                                Self::resolve_fallback_acl_context(target_uri.clone(), acl_chain)
                                    .await?
                            },
                            context.into_inner().into_with_arced_graph(),
                        )
                        .inspect_err(|_| {
                            error!("Error in resolving grants for nearest ipr");
                        })
                        .await
                }
                SlotRelationType::Auxiliary(aux_rel_type) => {
                    match aux_rel_type.target_access_resolution_role() {
                        // If policy-independent aux, then
                        // it is an ipr, and also no inheritance apply.
                        AuxAccessResolutionRole::Independent => {
                            info!("Aux resource with independent role is resolved to be nearest ipr. Uri: {}", target_uri.as_str());

                            engine
                                .resolve_access_control(
                                    own_item.acr.map(|acl| ResolvedAclContext {
                                        target_uri: target_uri.clone(),
                                        resolved_acl_subject_uri: own_item
                                            .res_slot
                                            .id()
                                            .uri
                                            .clone(),
                                        resolved_acl: acl,
                                    }),
                                    context.into_inner().into_with_arced_graph(),
                                )
                                .inspect_err(|_| {
                                    error!("Error in resolving grants for nearest ipr");
                                })
                                .await
                        }
                        AuxAccessResolutionRole::SubjectResource
                        | AuxAccessResolutionRole::SubjectResourceControl => {
                            let role = aux_rel_type.target_access_resolution_role();

                            info!(
                                "Reached an aux resource with dependent role. Role: {:?}, Subject res uri: {}",
                                role,
                                host_res_uri.as_str()
                            );

                            // Create access context for subject resource.
                            let mut subject_access_context = context.into_inner();
                            subject_access_context.set(&ns::acp::target, host_res_uri.deref());

                            // Then resolve grant for subject resource.
                            let subject_access_grants = Self::resolve_grants_with(
                                ResourceAccessContext::new_unchecked(
                                    host_res_uri.clone(),
                                    subject_access_context,
                                ),
                                acl_chain,
                                engine,
                                supported_access_modes.clone(),
                            )
                            .inspect_err(|_| {
                                error!("Error in resolving subject resource access grants.");
                            })
                            .await?
                            .access_grant_set;

                            if role == AuxAccessResolutionRole::SubjectResource {
                                // Resolve subject access grants as target's grants.
                                Ok(subject_access_grants)
                            } else {
                                // Resolve target's grants based on subject's `control` grant.
                                if subject_access_grants
                                    .iter()
                                    .any(|m| Term::eq(m.as_term(), ns::acl::Control))
                                {
                                    // If control grant available on subject,
                                    // then all grants are available on aux resource.
                                    Ok(supported_access_modes.as_ref().clone())
                                } else {
                                    // Or else no grants.
                                    Ok(HashSet::new())
                                }
                            }
                        }
                    }
                }
            },
        }?;

        Ok(AccessGrantResponse {
            res_slot: Some(own_item.res_slot),
            access_grant_set,
        })
    }

    async fn resolve_fallback_acl_context(
        target_uri: NormalAbsoluteHttpUri,
        mut acr_chain: SlotAcrChain<S, G, Arc<G>>,
    ) -> ProbResult<Option<ResolvedAclContext<G, Arc<G>>>> {
        while let Some(item_result) = acr_chain.next().await {
            let item = item_result.map_err(|e| {
                error!("Unknown io error in getting acr from prp. Error:\n {}", e);
                UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
            })?;

            match item.res_slot.slot_rev_link() {
                // Storage root.
                None => {
                    // Own acl is effective if exists.
                    // And is termination of fallback chain otherwise.
                    return Ok(item.acr.map(|acl| ResolvedAclContext {
                        target_uri,
                        resolved_acl_subject_uri: item.res_slot.id().uri.clone(),
                        resolved_acl: acl,
                    }));
                }
                Some(SlotRevLink {
                    target: _,
                    rev_rel_type,
                }) => match rev_rel_type {
                    // A contained ancestor
                    SlotRelationType::Contains => {
                        if let Some(acl) = item.acr {
                            // Own acl is effective if exists.
                            return Ok(Some(ResolvedAclContext {
                                target_uri,
                                resolved_acl_subject_uri: item.res_slot.id().uri.clone(),
                                resolved_acl: acl,
                            }));
                        } else {
                            // And continues fallback chain otherwise.
                            continue;
                        }
                    }
                    SlotRelationType::Auxiliary(aux_rel_type) => {
                        match aux_rel_type.target_access_resolution_role() {
                            AuxAccessResolutionRole::Independent => {
                                // Own acl is effective if exists.
                                // But breaks fallback chain otherwise.
                                return Ok(item.acr.map(|acl| ResolvedAclContext {
                                    target_uri,
                                    resolved_acl_subject_uri: item.res_slot.id().uri.clone(),
                                    resolved_acl: acl,
                                }));
                            }
                            AuxAccessResolutionRole::SubjectResource => {
                                // Own acl is not effective, but doesn't break fallback chain.
                                continue;
                            }
                            AuxAccessResolutionRole::SubjectResourceControl => {
                                // Own acl is not effective, and also breaks fallback chain.
                                return Ok(None);
                            }
                        }
                    }
                },
            }
        }

        Ok(None)
    }
}
