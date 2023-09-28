use std::{marker::PhantomData, ops::Deref, sync::Arc};

use acp::model::acr::{DAccessControlResource, HAccessControlResource};
use async_stream::stream;
use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, ProbResult};
use futures::{TryFutureExt, TryStreamExt};
use manas_access_control::model::{
    pdp::UNKNOWN_TARGET_RESOURCE,
    prp::{PolicyRetrievalPoint, SlotAcrChain, SlotAcrChainItem},
};
use manas_http::representation::{impl_::binary::BinaryRepresentation, Representation};
use manas_repo::{
    context::RepoContextual,
    service::resource_operator::{
        common::problem::{
            INVALID_EXISTING_REPRESENTATION_STATE, INVALID_RDF_SOURCE_REPRESENTATION,
            UNSUPPORTED_OPERATION,
        },
        reader::rep_preferences::{
            range_negotiator::impl_::CompleteRangeNegotiator, ContainerRepresentationPreference,
            RepresentationPreferences,
        },
    },
};
use manas_space::resource::uri::SolidResourceUri;
use rdf_dynsyn::syntax::invariant::parsable::DynSynParsableSyntax;
use rdf_utils::model::{
    description::SimpleDescription, graph::InfallibleMutableGraph, handle::Handle, quad::ArcQuad,
    term::ArcTerm, triple::ArcTriple,
};
use sophia_api::{graph::SetGraph, term::Term};
use tracing::{error, info};

use super::resource_operator::reader::ODRResourceReader;
use crate::{
    context::ODRContext,
    resource_context::{invariant::ODRClassifiedResourceContext, ODRResourceContext},
    setup::{aux_rep_policy::ODRAuxResourcePolicyExt, ODRSetup},
    status_token::{
        inputs::ODRResourceStatusTokenInputs, variant::ODRExistingRepresentedResourceToken,
        ODRBaseResourceStatusToken,
    },
    OpendalRepo,
};

/// Alias for `DAccessControlResource` backed by an arced graph.
pub type ArcedAcr<G> = DAccessControlResource<G, Arc<G>>;

/// An implementation of access control [`PolicyRetrievalPoint`] backed by odr.
#[derive(Debug, Clone)]
pub struct ODRPolicyRetrievalPoint<Setup: ODRSetup, G> {
    /// Repo context.
    repo_context: Arc<ODRContext<Setup>>,
    _phantom: PhantomData<G>,
}

impl<Setup, G> RepoContextual for ODRPolicyRetrievalPoint<Setup, G>
where
    Setup: ODRSetup,
{
    type Repo = OpendalRepo<Setup>;

    #[inline]
    fn new_with_context(repo_context: Arc<ODRContext<Setup>>) -> Self {
        Self {
            repo_context,
            _phantom: PhantomData,
        }
    }

    #[inline]
    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }
}

impl<Setup, G> PolicyRetrievalPoint for ODRPolicyRetrievalPoint<Setup, G>
where
    Setup: ODRSetup,
    for<'x> G:
        InfallibleMutableGraph + SetGraph + Default + Extend<ArcTriple> + Send + Sync + 'static,
{
    type StSpace = Setup::StSpace;

    type Graph = G;

    type WGraph = Arc<G>;

    #[tracing::instrument(skip_all, name = "ODRPolicyRetrievalPoint::retrieve", fields(res_uri))]
    fn retrieve(
        &self,
        res_uri: SolidResourceUri,
        // TODO support or remove.
        _deduced_containment_is_sufficient: bool,
    ) -> ProbFuture<'static, SlotAcrChain<Self::StSpace, Self::Graph, Self::WGraph>> {
        let repo_context = self.repo_context.clone();

        Box::pin(async move {
            // Check backend capabilities.
            ODRResourceReader::<Setup>::ensure_backend_caps(&repo_context)?;

            // Decode resource context.
            let res_context = ODRClassifiedResourceContext::new(Arc::new(
                ODRResourceContext::try_new(res_uri, repo_context).map_err(|e| {
                    error!("Error in decoding context for resource. Error:\n {}", e);
                    UNKNOWN_TARGET_RESOURCE
                        .new_problem_builder()
                        .source(e)
                        .finish()
                })?,
            ));

            // Resolve known acl rel type.
            let kn_acl_rel_type =
                <Setup::AuxResourcePolicy as ODRAuxResourcePolicyExt>::known_acl_rel_type()
                    .ok_or_else(|| {
                        error!("Repo was not configured to support acl aux resources.");
                        UNSUPPORTED_OPERATION.new_problem()
                    })?;

            // Get resource status token inputs.
            let res_status_token =
                ODRBaseResourceStatusToken::try_current(res_context.clone().into_inner())
                    .await
                    .map_err(|e| {
                        error!("Error in resolving resource status inputs. Error:\n {}", e);
                        UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                    })?;

            // ensure that resource exists.
            if !res_status_token.is_of_existing() {
                error!("Resource doesn't exists.");
                return Err(UNKNOWN_TARGET_RESOURCE.new_problem());
            }

            Ok(Box::pin(stream! {
                let mut opt_iter_res_context = Some(res_context.into_inner());

                while let Some(iter_res_context) = opt_iter_res_context {

                    if let Some(acl_res_context) = iter_res_context.aux_resource_context(kn_acl_rel_type.clone()) {
                        yield Self::retrieve_acl_graph(ODRClassifiedResourceContext::new(Arc::new(acl_res_context))).map_ok(|acr| {
                            SlotAcrChainItem {
                                acr,
                                res_slot: iter_res_context.slot().clone(),
                            }
                        }).await
                    } else {
                        yield Ok(SlotAcrChainItem {
                                acr: None,
                                res_slot: iter_res_context.slot().clone(),
                            });
                    }

                    opt_iter_res_context = iter_res_context.host_resource_context().map(Arc::new);
                }
            })
                as SlotAcrChain<
                    Setup::StSpace,
                    Self::Graph,
                    Arc<Self::Graph>,
                >)
        })
    }
}

impl<Setup, G> ODRPolicyRetrievalPoint<Setup, G>
where
    Setup: ODRSetup,
    for<'x> G:
        InfallibleMutableGraph + SetGraph + Default + Extend<ArcTriple> + Send + Sync + 'static,
{
    async fn retrieve_acl_graph(
        acl_res_context: ODRClassifiedResourceContext<Setup>,
    ) -> ProbResult<Option<ArcedAcr<G>>> {
        let acl_status_token_inputs =
            ODRResourceStatusTokenInputs::try_current(acl_res_context.clone())
                .await
                .map_err(|e| {
                    error!("Error in resolving acl status inputs. Error:\n {}", e);
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })?;

        // Get represented status status token.
        let acl_er_token = if let Ok(token) =
            ODRExistingRepresentedResourceToken::try_from(acl_status_token_inputs)
        {
            token
        } else {
            info!("Acl is not represented.");
            return Ok(None);
        };

        // Get representation.
        let rep: BinaryRepresentation = acl_er_token
            .try_resolve_representation(RepresentationPreferences {
                container_rep_preference: ContainerRepresentationPreference::Minimal,
                non_container_rep_range_negotiator: Box::new(CompleteRangeNegotiator),
            })
            .await
            .map_err(ODRResourceReader::<Setup>::map_state_resolution_err)?;

        let parsable_syntax = rep
            .metadata()
            .rdf_syntax::<DynSynParsableSyntax>()
            .ok_or_else(|| {
                error!("Acr rep content type is not quads parsable.");
                INVALID_EXISTING_REPRESENTATION_STATE.new_problem()
            })?
            .value;

        //  Parse graph.
        let rep_base_uri = rep.base_uri().as_ref().map(Into::into);
        let graph = acl_res_context
            .repo_context()
            .config
            .dynsyn_factories
            .parser
            .parse_quads_from_bytes_stream::<_, ArcTerm>(
                rep.into_streaming().into_parts().0.stream,
                rep_base_uri,
                parsable_syntax,
            )
            .await
            .map_ok(|q: ArcQuad| q.0)
            .try_collect::<G>()
            .await
            .map_err(|_| {
                error!("Error in parsing acl as rdf source doc.");
                INVALID_RDF_SOURCE_REPRESENTATION.new_problem()
            })?;

        Ok(Some(DAccessControlResource::new(
            HAccessControlResource::try_new(acl_res_context.uri().deref().into_term())
                .expect("Must be valid term for handle"),
            Arc::new(graph),
        )))
    }
}
