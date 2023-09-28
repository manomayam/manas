//! I define an implementation of [`PolicyDecisionPoint`] that
//! confirms to `ACP` specification.
//!

use std::{collections::HashSet, fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use acp::{
    engine::AcpEngine,
    model::{
        access_mode::{HAccessMode, H_APPEND, H_CONTROL, H_READ, H_WRITE},
        acr::DAccessControlResource,
        attribute::HAttribute,
    },
};
use async_recursion::async_recursion;
use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, ProbResult};
use futures::{StreamExt, TryFutureExt};
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

use crate::model::{
    pdp::{AccessGrantResponse, PolicyDecisionPoint, ResourceAccessContext, INVALID_PRP_RESPONSE},
    prp::SlotAcrChain,
};

// fn sy<T: Send>(v: T) -> T {
//     v
// }

/// An implementation of [`PolicyDecisionPoint`] that confirms to `ACP` specification.
#[derive(Clone)]
pub struct AcpDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + Default + Clone,
{
    /// Acp engine.
    engine: AcpEngine<G, Arc<G>>,

    ///Supported access modes.
    supported_access_modes: Arc<HashSet<HAccessMode<ArcTerm>>>,

    ///Supported attributes.
    supported_attrs: Arc<HashSet<HAttribute<ArcTerm>>>,

    _phantom: PhantomData<fn(S)>,
}

impl<S, G> Debug for AcpDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + Default + Clone + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpDecisionPoint").finish()
    }
}

impl<S, G> Default for AcpDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + Default + Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        let engine = AcpEngine::default();
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

impl<S, G> AcpDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + Default + Clone,
{
    /// Create a new [`AcpDecisionPoint`] with given params.
    #[inline]
    pub fn new(
        engine: AcpEngine<G, Arc<G>>,
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

impl<S, G> PolicyDecisionPoint for AcpDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + SetGraph + Debug + Default + Clone + Send + Sync + 'static,
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

    #[tracing::instrument(skip_all, name = "AcpDecisionPoint::resolve_grants")]
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

impl<S, G> AcpDecisionPoint<S, G>
where
    S: SolidStorageSpace,
    G: InfallibleMutableGraph + SetGraph + Debug + Default + Clone + Send + Sync + 'static,
{
    /// Resolve grants with given context and acr chain.
    #[async_recursion]
    async fn resolve_grants_with(
        context: ResourceAccessContext<G>,
        mut acr_chain: SlotAcrChain<S, G, Arc<G>>,
        engine: AcpEngine<G, Arc<G>>,
        supported_access_modes: Arc<HashSet<HAccessMode<ArcTerm>>>,
    ) -> ProbResult<AccessGrantResponse<S>> {
        let target_uri = context.target_uri().clone();

        // Retrieve own item.
        let own_item = acr_chain
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
                        own_item.acr,
                        // No ancestor acrs for storage root.
                        vec![],
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
                            own_item.acr,
                            Self::resolve_inherited_acrs(acr_chain).await?,
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
                                    own_item.acr,
                                    // No ancestor acrs for aux resource with independent role.
                                    vec![],
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
                                acr_chain,
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

    async fn resolve_inherited_acrs(
        mut acr_chain: SlotAcrChain<S, G, Arc<G>>,
    ) -> ProbResult<Vec<Option<DAccessControlResource<G, Arc<G>>>>> {
        let mut inherited_acrs = Vec::new();

        while let Some(item_result) = acr_chain.next().await {
            let item = item_result.map_err(|e| {
                error!("Unknown io error in getting acr from prp. Error:\n {}", e);
                UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
            })?;

            match item.res_slot.slot_rev_link() {
                // Storage root.
                None => {
                    // Item's acr is effective if exist, and
                    // reached end of inheritance chain.
                    inherited_acrs.push(item.acr);
                    break;
                }
                Some(SlotRevLink {
                    target: _,
                    rev_rel_type,
                }) => match rev_rel_type {
                    // A contained ancestor
                    SlotRelationType::Contains => {
                        // Item's acr is effective if exist,
                        // inheritance chain will continue.
                        inherited_acrs.push(item.acr);
                        continue;
                    }
                    SlotRelationType::Auxiliary(aux_rel_type) => {
                        match aux_rel_type.target_access_resolution_role() {
                            AuxAccessResolutionRole::Independent => {
                                // Item's acr is effective if exist,
                                // inheritance chain will break.
                                inherited_acrs.push(item.acr);
                                break;
                            }
                            AuxAccessResolutionRole::SubjectResource => {
                                // Item's acr is ineffective, and
                                // Inheritance chain will continue.
                                inherited_acrs.push(None);
                                continue;
                            }
                            AuxAccessResolutionRole::SubjectResourceControl => {
                                // Item's acr is ineffective, and
                                // inheritance chain will break.
                                inherited_acrs.push(None);
                                break;
                            }
                        }
                    }
                },
            }
        }

        Ok(inherited_acrs)
    }
}
