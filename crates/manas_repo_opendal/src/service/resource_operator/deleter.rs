//! I provide an implementation of [`ResourceDeleter`] for ODR.
//!

use std::{marker::PhantomData, task::Poll};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt};
use futures::{StreamExt, TryFutureExt};
use manas_http::representation::impl_::common::data::quads_stream::BoxQuadsStream;
use manas_repo::service::resource_operator::{
    common::{
        preconditions::{KEvaluatedRepValidators, KPreconditionsEvalResult},
        problem::{PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_OPERATION},
        status_token::RepoResourceStatusTokenBase,
    },
    deleter::{
        ResourceDeleteRequest, ResourceDeleteResponse, ResourceDeleter,
        DELETE_TARGETS_NON_EMPTY_CONTAINER,
    },
};
use tower::Service;
use tracing::{error, info, warn};

use crate::{
    object_store::{
        backend::{BackendExtraCapability, ODRObjectStoreBackend},
        object::invariant::ODRNamespaceObjectExt,
    },
    service::resource_operator::common::{
        remnants::purge_remnants, status_token::inputs::container_index::ODRContainerIndexInputs,
    },
    setup::ODRSetup,
    OpendalRepo,
};

/// An implementation of [`ResourceDeleter``] for ODR.
#[derive(Debug, Clone)]
pub struct ODRResourceDeleter<Setup> {
    _phantom: PhantomData<Setup>,
}

impl<Setup> Default for ODRResourceDeleter<Setup> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<Setup: ODRSetup> Service<ResourceDeleteRequest<OpendalRepo<Setup>>>
    for ODRResourceDeleter<Setup>
{
    type Response = ResourceDeleteResponse<OpendalRepo<Setup>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(skip_all, name = "ODRResourceDeleter::call", fields(req))]
    fn call(&mut self, req: ResourceDeleteRequest<OpendalRepo<Setup>>) -> Self::Future {
        Box::pin(async move {
            let token = req.tokens.res_token;
            let repo_context = token.repo_context().clone();

            // Ensure backend has required capabilities.
            let backend_caps = repo_context.backend_caps();

            if !(backend_caps.stat && backend_caps.read && backend_caps.list && backend_caps.write)
            {
                error!(
                    "ODR backend doesn't have required capabilities to support delete operation."
                );
                return Err(UNSUPPORTED_OPERATION.new_problem());
            }

            // Get extra capabilities of the backend.
            let backend_extra_caps = repo_context.object_store.backend.extra_caps();

            let is_flat_backend =
                backend_extra_caps.contains(BackendExtraCapability::HasIndependentDirObjects);

            let prev_status_inputs = token.status_inputs();
            let res_context = prev_status_inputs.res_context.clone();

            // Check preconditions.
            let rep_validators = token.resolve_rep_validators();

            // Evaluate preconditions.
            let pc_eval_result = req.preconditions.evaluate(Some(&rep_validators));

            // Return error if preconditions are not satisfied.
            if !pc_eval_result.are_satisfied() {
                return Err(PRECONDITIONS_NOT_SATISFIED
                    .new_problem_builder()
                    .extend_with::<KPreconditionsEvalResult>(pc_eval_result)
                    .extend_with::<KEvaluatedRepValidators>(Some(rep_validators))
                    .finish());
            }

            // If res is a container, ensure it is empty.
            if let Some(c_res_context) = res_context.as_left_classified() {
                let mut container_index_quads: BoxQuadsStream = (ODRContainerIndexInputs {
                    c_res_context: c_res_context.clone(),
                })
                .resolve()
                .inspect_ok(|_| info!("Success in getting container index stream."))
                .await
                .map_err(|e| {
                    error!("Error in getting container index stream.");
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })?;

                // If there are no quad results, then container is empty.
                let is_empty_container = container_index_quads.next().await.is_none();

                // If not an empty container, return error.
                if !is_empty_container {
                    error!("Delete target container is not empty.");
                    return Err(DELETE_TARGETS_NON_EMPTY_CONTAINER.new_problem());
                }
            }

            if res_context.is_left_classified() && !is_flat_backend {
                // If backend doesn't have independent dir objects,
                // and the resource is a container, then
                // delete operation is not atomic.
                // We will try to delete all objects under the
                // namespace of base object.
                // Aux tree of container will also be deleted,
                // as aux namespace is sub namespace of base namespace.

                res_context
                .assoc_odr_object_map()
                .base_object()
                .delete_recursive()
                .inspect_ok(|_| info!("Success in deleting container's base namespace."))
                .await
                .map_err(|e| {
                    error!("Error in deleting container's base namespace. Operation failed non-atomically. Error:\n {}", e);
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })?;
            } else {
                // Else operation will be atomic.

                // First delete base object.
                let indicator_object = res_context.assoc_odr_object_map().base_object();

                indicator_object
                    .delete()
                    .inspect_ok(|_| info!("Success in deleting assoc base object."))
                    .await
                    .map_err(|e| {
                        error!("Error in deleting assoc base object. Error:\n {}", e);
                        UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                    })?;

                // Then try purge any remnants.
                let _ = purge_remnants(&res_context)
                    .inspect_err(|e| warn!("Error in purging remnants. Error:\n {}", e))
                    .await;

                // Delete operation considered success after deleting indicator,
                // even if purging of remnants failed.
                // Remnants will be ignored in resource status resolution.
            }

            // If res is contained, Update host container index timestamp.
            // Explicit update is only required in flat object spaces.
            if_chain::if_chain! {
                if is_flat_backend;
                if res_context.slot().is_contained_slot();
                if let Some(host_container_context) = res_context.host_resource_context();

                then {
                    let _ = host_container_context
                        .assoc_odr_object_map()
                        .base_object()
                        .as_left_classified()
                        .expect("Must be namespace object.")
                        .create()
                        .inspect_err(|_| {
                            warn!("Io error in updating host container's base object timestamp.")
                        })
                        .await;
                }
            }

            Ok(ResourceDeleteResponse {
                deleted_res_slot: res_context.slot().clone(),
                deleted_aux_res_links: res_context.supported_aux_links().collect(),
                extensions: Default::default(),
            })
        })
    }
}

impl<Setup: ODRSetup> ResourceDeleter for ODRResourceDeleter<Setup> {
    type Repo = OpendalRepo<Setup>;
}
