//! I provide an implementation of [`ResourceUpdater`] for ODR.
//!

use std::{marker::PhantomData, task::Poll};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt};
use futures::TryFutureExt;
use manas_http::{
    header::common::media_type::APPLICATION_JSON,
    representation::{impl_::binary::BinaryRepresentation, Representation},
};
use manas_repo::service::resource_operator::{
    common::{
        preconditions::{KEvaluatedRepValidators, KPreconditionsEvalResult},
        problem::{PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_OPERATION},
        rep_update_action::RepUpdateAction,
    },
    updater::{ResourceUpdateRequest, ResourceUpdateResponse, ResourceUpdater},
};
use tower::Service;
use tracing::{error, info};

use crate::{
    object_store::{
        backend::{BackendExtraCapability, ODRObjectStoreBackend},
        object::invariant::ODRFileObjectExt,
        object_space::assoc::rel_type::sidecar::SidecarRelType,
    },
    service::resource_operator::common::status_token::{
        inputs::altfm::AltFatMetadata,
        variant::{decode_rep_content_type, ODRBaseExistingResourceToken},
    },
    setup::ODRSetup,
    OpendalRepo,
};

/// An implementation of [`ResourceUpdater``] for ODR.
#[derive(Debug, Clone)]
pub struct ODRResourceUpdater<Setup> {
    _phantom: PhantomData<Setup>,
}

impl<Setup> Default for ODRResourceUpdater<Setup> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<Setup: ODRSetup> Service<ResourceUpdateRequest<OpendalRepo<Setup>>>
    for ODRResourceUpdater<Setup>
{
    type Response = ResourceUpdateResponse;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ResourceUpdateRequest<OpendalRepo<Setup>>) -> Self::Future {
        Box::pin(async move {
            let token: ODRBaseExistingResourceToken<Setup> = req.tokens.res_token.into();

            let repo_context = token.repo_context().clone();

            // Ensure backend has required capabilities.
            let backend_caps = repo_context.backend_caps();

            if !(backend_caps.stat && backend_caps.read && backend_caps.list && backend_caps.write)
            {
                error!(
                    "ODR backend doesn't have required capabilities to support update operation."
                );
                return Err(UNSUPPORTED_OPERATION.new_problem());
            }

            // Get extra capabilities of the backend.
            let backend_extra_caps = repo_context.object_store.backend.extra_caps();

            let is_cty_capable_backend = backend_extra_caps
                .contains(BackendExtraCapability::SupportsNativeContentTypeMetadata);

            let prev_status_inputs = token.status_inputs();
            let res_context = prev_status_inputs.res_context.clone();

            let prev_rep_validators = token.resolve_rep_validators();

            // Evaluate preconditions.
            let pc_eval_result = req.preconditions.evaluate(prev_rep_validators.as_ref());

            // Return error if preconditions are not satisfied.
            if !pc_eval_result.are_satisfied() {
                return Err(PRECONDITIONS_NOT_SATISFIED
                    .new_problem_builder()
                    .extend_with::<KPreconditionsEvalResult>(pc_eval_result)
                    .extend_with::<KEvaluatedRepValidators>(prev_rep_validators)
                    .finish());
            }

            // Resolve effective new rep.
            let effective_new_rep: BinaryRepresentation =
                if let RepUpdateAction::SetWith(rep) = req.rep_update_action {
                    rep
                } else {
                    error!("opendal repo doesn't support patch operation natively.");
                    return Err(UNSUPPORTED_OPERATION
                        .new_problem_builder()
                        .message("opendal repo doesn't support patch operation natively.")
                        .finish());
                };

            let assoc_odr_obj_map = res_context.as_ref().as_ref().assoc_odr_object_map();

            let prev_resolved_alt_metadata = token.as_represented().and_then(|er_token| {
                er_token
                    .try_resolve_effective_alt_metadata()
                    // If invalid altfm object, then set
                    // resolved to `None`, and ignore
                    // invalidity. TODO should warn.
                    .unwrap_or(None)
            });

            // Decode default content type from uri.
            let decoded_content_type =
                decode_rep_content_type::<Setup>(res_context.as_ref().as_ref());

            let new_rep_content_type = effective_new_rep.metadata().content_type().clone();

            // Check if actual content-type of new rep is
            // diverging from that of uri decoded.
            let is_diverging_new_content_type =
                new_rep_content_type.essence_str() != decoded_content_type.essence_str();

            let should_create_alt_fm = (is_diverging_new_content_type && !is_cty_capable_backend)
            // If altfm object already exist, bring it to consistency.
            || prev_status_inputs.altfm_obj_content.is_some();

            // Persist altfm first,
            if should_create_alt_fm {
                let new_altfm = AltFatMetadata::resolve_new(
                    prev_resolved_alt_metadata,
                    if res_context.is_left_classified() {
                        // For container
                        prev_status_inputs.altcontent_obj_metadata.clone()
                    } else {
                        prev_status_inputs.base_obj_metadata.clone()
                    },
                    new_rep_content_type.clone(),
                );

                assoc_odr_obj_map
                    .sidecar_object(SidecarRelType::AltFatMeta)
                    .write(
                        serde_json::to_string(&new_altfm).expect("Must be valid"),
                        &APPLICATION_JSON,
                    )
                    .inspect_ok(|_| {
                        info!("Success in writing new altfm.");
                    })
                    .await
                    .map_err(|e| {
                        error!("Unknown io error in writing new altfm. Error:\n {}", e);
                        UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                    })?;
            }

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

            // Write rep content.
            content_obj
                .write_streaming(
                    effective_new_rep.into_streaming().into_parts().0.stream,
                    &new_rep_content_type,
                )
                .inspect_ok(|_| info!("Success in writing new rep data"))
                .await
                .map_err(|e| {
                    error!("Unknown io error in writing new rep data. Error:\n {}", e);
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })?;

            Ok(ResourceUpdateResponse {
                extensions: Default::default(),
            })
        })
    }
}

impl<Setup: ODRSetup> ResourceUpdater for ODRResourceUpdater<Setup> {
    type Repo = OpendalRepo<Setup>;
}
