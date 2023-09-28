//! I provide an implementation of [`ResourceCreator`] for ODR.
//!

use std::{marker::PhantomData, task::Poll};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt};
use futures::TryFutureExt;
use manas_http::{
    header::common::media_type::{MediaType, APPLICATION_JSON},
    representation::{
        impl_::binary::BinaryRepresentation, metadata::KCompleteContentLength, Representation,
    },
};
use manas_repo::service::resource_operator::{
    common::{
        preconditions::{KEvaluatedRepValidators, KPreconditionsEvalResult},
        problem::{PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_OPERATION, URI_POLICY_VIOLATION},
        rep_update_action::RepUpdateAction,
    },
    creator::{ResourceCreateRequest, ResourceCreateResponse, ResourceCreator},
};
use manas_space::resource::slot_rev_link::SlotRevLink;
use tower::Service;
use tracing::{error, info, warn};
use typed_record::TypedRecord;

use crate::{
    object_store::{
        backend::{BackendExtraCapability, ODRObjectStoreBackend},
        object::invariant::{ODRFileObjectExt, ODRNamespaceObjectExt},
        object_space::assoc::rel_type::sidecar::SidecarRelType,
    },
    resource_context::invariant::ODRClassifiedResourceContext,
    service::resource_operator::common::{
        remnants::purge_remnants,
        status_token::{
            inputs::{
                altfm::{AltFatMetadata, AltMetadata},
                ODRResourceStatusTokenInputs,
            },
            variant::decode_rep_content_type,
        },
    },
    setup::ODRSetup,
    OpendalRepo,
};

/// An implementation of [`ResourceCreator``] for ODR.
#[derive(Debug, Clone)]
pub struct ODRResourceCreator<Setup> {
    _phantom: PhantomData<Setup>,
}

impl<Setup> Default for ODRResourceCreator<Setup> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<Setup: ODRSetup> Service<ResourceCreateRequest<OpendalRepo<Setup>>>
    for ODRResourceCreator<Setup>
{
    type Response = ResourceCreateResponse<OpendalRepo<Setup>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(skip_all, name = "ODRResourceCreator::call", fields(req))]
    fn call(&mut self, req: ResourceCreateRequest<OpendalRepo<Setup>>) -> Self::Future {
        Box::pin(async move {
            let repo_context = req.tokens.repo_context().clone();

            let (res_token, host_token) = req.tokens.into_parts();

            // Ensure backend has required capabilities.
            let backend_caps = repo_context.backend_caps();

            if !(backend_caps.stat && backend_caps.read && backend_caps.list && backend_caps.write)
            {
                error!(
                    "ODR backend doesn't have required capabilities to support create operation."
                );
                return Err(UNSUPPORTED_OPERATION.new_problem());
            }

            // Get extra capabilities of the backend.
            let backend_extra_caps = repo_context.object_store.backend.extra_caps();

            // Flat backend has independent dir objects.
            let is_flat_backend =
                backend_extra_caps.contains(BackendExtraCapability::HasIndependentDirObjects);

            // Ensure context is resolvable.
            let res_context = if let Some(status_inputs) = res_token.own_status_inputs() {
                status_inputs.res_context.clone()
            } else {
                error!("Context is not resolvable for the resource.",);
                return Err(URI_POLICY_VIOLATION.new_problem());
            };

            let host_res_context = host_token.status_inputs().res_context.clone();

            // Ensure supplied res kind doesn't contradict encoded.
            if res_context.kind() != req.resource_kind {
                error!("Supplied res kind contradicts with encoded semantics.");
                return Err(URI_POLICY_VIOLATION.new_problem());
            }

            // Ensure slot relation is containment.
            // ODR supports explicit creation of only
            // contained resources.
            if !req.slot_rev_rel_type.is_contains() {
                error!("ODR invariant error. Aux resource must had been minted already.");
                return Err(UNSUPPORTED_OPERATION.new_problem());
            }

            // Get encoded slot rev link.
            let encoded_slot_rev_link: &SlotRevLink<Setup::StSpace> =
                res_context.slot().slot_rev_link().ok_or_else(|| {
                    // If no slot rev link, then it is a storage root.
                    // Storage root must have been already existing.
                    error!("Repo is not initialized. Storage root doesn't exists.");
                    UNSUPPORTED_OPERATION.new_problem()
                })?;

            // Ensure supplied slot rev link param matches
            // with uri encoded one.
            if (&encoded_slot_rev_link.target != host_res_context.uri())
                || (encoded_slot_rev_link.rev_rel_type != req.slot_rev_rel_type)
            {
                error!("Encoded slot rev link doesn't matched with supplied params.");
                return Err(URI_POLICY_VIOLATION.new_problem());
            }

            // Resolve host container's rep validators.
            let host_container_rep_validators = host_token.resolve_rep_validators();

            // Evaluate preconditions.
            let pc_eval_result = req
                .host_preconditions
                .evaluate(Some(&host_container_rep_validators));

            // Return error, if preconditions are not satisfied.
            if !pc_eval_result.are_satisfied() {
                return Err(PRECONDITIONS_NOT_SATISFIED
                    .new_problem_builder()
                    .extend_with::<KPreconditionsEvalResult>(pc_eval_result)
                    .extend_with::<KEvaluatedRepValidators>(Some(host_container_rep_validators))
                    .finish());
            }

            // Purge any previous remnants.
            purge_remnants(res_context.as_ref())
                .inspect_ok(|_| info!("Remnants purging succeeded"))
                .map_err(|e| {
                    error!("Io error in purging remnants. Error:\n {}", e);
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })
                .await?;

            // As resource doesn't exists, and remnants are
            // purged, new resource status inputs will be
            let mut res_status_inputs =
                ODRResourceStatusTokenInputs::new_non_existing(res_context.clone());
            res_status_inputs.slot_path_is_represented = true;

            // Resolve effective rep.
            let effective_rep: BinaryRepresentation =
                if let RepUpdateAction::SetWith(rep) = req.rep_update_action {
                    rep
                } else {
                    error!("opendal repo doesn't support patch operation natively.");
                    return Err(UNSUPPORTED_OPERATION
                        .new_problem_builder()
                        .message("opendal repo doesn't support patch operation natively.")
                        .finish());
                };

            let effective_rep_content_type = effective_rep.metadata().content_type().clone();

            // Decode default content type from uri.
            let decoded_content_type = decode_rep_content_type::<Setup>(res_context.as_ref());

            // Check if actual content-type of rep is
            // diverging from that of uri decoded.
            let is_diverging_content_type =
                effective_rep_content_type.essence_str() != decoded_content_type.essence_str();

            // First create altfm if required.
            let _altfm_created =
                Self::create_altfm(&res_context, &effective_rep_content_type).await?;

            let assoc_odr_obj_map = res_context.as_ref().as_ref().assoc_odr_object_map();

            // Resolve content odr object.
            let content_obj = if res_context.is_left_classified() {
                // If resource is a container, assoc content object will be alt-object.
                assoc_odr_obj_map.sidecar_object(SidecarRelType::AltContent)
            } else {
                // If resource is a non-container,
                assoc_odr_obj_map
                    .base_object()
                    .as_right_classified()
                    .expect("Base object must be file object for non-containers")
            };

            // Check if user supplied rep is a trivial container rep.
            let is_trivial_container_us_rep = res_context.is_left_classified()
                && !is_diverging_content_type
                && effective_rep
                    .metadata()
                    .get_rv::<KCompleteContentLength>()
                    .map_or(false, |content_length| content_length.0 == 0);

            // Write rep content, if it is not trivial.
            if !is_trivial_container_us_rep {
                if let Err(e) = content_obj
                    .write_streaming(
                        effective_rep.into_streaming().into_parts().0.stream,
                        &effective_rep_content_type,
                    )
                    .inspect_ok(|_| info!("Success in writing rep content."))
                    .await
                {
                    error!("Error in writing rep content.. Error:\n {}", e);

                    // Try clean remnants.
                    let _ = purge_remnants(&res_context).await;
                    return Err(UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish());
                }
            }

            // Create indicator object, if resource is a container.
            if res_context.is_left_classified() {
                let indicator_obj = assoc_odr_obj_map
                    .base_object()
                    .as_left_classified()
                    .expect("Container's assoc base object must be a namespace object.");

                if let Err(e) = indicator_obj
                    .create()
                    .inspect_ok(|_| {
                        info!("Container's assoc indicator object creation successful.")
                    })
                    .await
                {
                    error!(
                        "Error in creating container's assoc indicator object. Error:\n {}",
                        e
                    );
                    // Try clean remnants.
                    let _ = purge_remnants(&res_context).await;
                    return Err(UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish());
                }
            }

            // Update host container index timestamp.
            // On hierarchical obj spaces, container rep object
            // timestamp will be automatically updated
            // on adding or removing a member.
            // For flat obj spaces, we overwrite base object of
            // container with empty content.
            // As we use alt object for persisting actual
            // container rep content,
            // this doesn't overwrite effective content.
            if is_flat_backend {
                let _ = host_res_context
                    .as_ref()
                    .assoc_odr_object_map()
                    .base_object()
                    .as_left_classified()
                    .expect("Must be namespace object.")
                    .create()
                    .inspect_err(|_| {
                        warn!("Io error in updating host container's indicator object timestamp.")
                    })
                    .await;
            }

            // Aux resources considered minted with out
            // associated representations.

            Ok(ResourceCreateResponse {
                created_resource_slot: res_context.slot().clone(),
                extensions: Default::default(),
            })
        })
    }
}

impl<Setup: ODRSetup> ODRResourceCreator<Setup> {
    // Create alt fat metadata if necessary.
    async fn create_altfm(
        res_context: &ODRClassifiedResourceContext<Setup>,
        effective_rep_content_type: &MediaType,
    ) -> Result<bool, Problem> {
        let res_context = res_context.as_ref();

        let backend_extra_caps = res_context
            .as_ref()
            .repo_context()
            .object_store
            .backend
            .extra_caps();

        // Check if backend is flat.
        let is_flat_backend =
            backend_extra_caps.contains(BackendExtraCapability::HasIndependentDirObjects);

        // Check if backend is native cty capable.
        let is_cty_capable_backend =
            backend_extra_caps.contains(BackendExtraCapability::SupportsNativeContentTypeMetadata);

        // Decode default content type from uri.
        let decoded_content_type = decode_rep_content_type::<Setup>(res_context.as_ref());

        // Check if actual content-type of rep is
        // diverging from that of uri decoded.
        let is_diverging_content_type =
            effective_rep_content_type.essence_str() != decoded_content_type.essence_str();

        let should_create_alt_fm = is_diverging_content_type && !is_cty_capable_backend;

        // Return if no need to create alt fm.
        if !should_create_alt_fm {
            return Ok(false);
        }

        let alt_fm = AltFatMetadata {
            live: AltMetadata {
                content_type: Some(effective_rep_content_type.clone()),
            },
            prev_backup: None,
        };

        // Persist altfm.
        if let Err(e) = res_context
            .assoc_odr_object_map()
            .sidecar_object(SidecarRelType::AltFatMeta)
            .write(
                serde_json::to_string(&alt_fm).expect("Must be valid"),
                &APPLICATION_JSON,
            )
            .inspect_ok(|_| {
                info!("Success in writing alt fm.");
            })
            .await
        {
            error!("Unknown io error in writing alt fm.");
            if !is_flat_backend {
                // Failed op may leave active remnants in
                // backends with true dirs.
                let _ = purge_remnants(res_context).await;
                // As any possible remnants can be active, op is
                // non-atomic for non-flat backends.
            }
            return Err(UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish());
        };

        Ok(true)
    }
}

impl<Setup: ODRSetup> ResourceCreator for ODRResourceCreator<Setup> {
    type Repo = OpendalRepo<Setup>;
}
