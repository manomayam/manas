use std::{sync::Arc, task::Poll};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem};
use manas_repo::{
    context::{RepoContext, RepoContextual},
    service::{
        initializer::{RepoInitializer, INVALID_STORAGE_ROOT_URI},
        resource_operator::common::problem::UNSUPPORTED_OPERATION,
    },
};
use manas_space::SolidStorageSpace;
use tower::Service;
use tracing::error;

use crate::{
    context::ODRContext,
    object_store::object::invariant::ODRNamespaceObjectExt,
    resource_context::{invariant::ODRClassifiedResourceContext, ODRResourceContext},
    setup::ODRSetup,
    OpendalRepo,
};

/// An implementation of [`RepoInitializer`] for opendal repo.
#[derive(Debug, Clone)]
pub struct ODRInitializer<Setup: ODRSetup> {
    /// Context of the opendal repo.
    pub repo_context: Arc<ODRContext<Setup>>,
}

impl<Setup: ODRSetup> RepoContextual for ODRInitializer<Setup> {
    type Repo = OpendalRepo<Setup>;

    #[inline]
    fn new_with_context(context: Arc<ODRContext<Setup>>) -> Self {
        Self {
            repo_context: context,
        }
    }

    #[inline]
    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }
}

impl<Setup: ODRSetup> Service<()> for ODRInitializer<Setup> {
    type Response = bool;

    type Error = Problem;

    type Future = ProbFuture<'static, bool>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Always ready.
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "ODRInitializer::call")]
    fn call(&mut self, _params: ()) -> Self::Future {
        let repo_context = self.repo_context.clone();
        Box::pin(async move {
            // Check backend capabilities.
            let backend_caps = repo_context.backend_caps();

            if !(backend_caps.read && backend_caps.list) {
                error!("Backend doesn't have read, list capabilities to init.");
                return Err(UNSUPPORTED_OPERATION.new_problem());
            }

            // Try get storage root's context.
            let stroot_context = ODRClassifiedResourceContext::new(Arc::new(
                ODRResourceContext::try_new(
                    repo_context.storage_space().root_res_uri().clone(),
                    repo_context,
                )
                .map_err(|e| {
                    error!(
                        "Error in decoding resource context for storage root. Error:\n {}",
                        e
                    );
                    INVALID_STORAGE_ROOT_URI
                        .new_problem_builder()
                        .source(e)
                        .finish()
                })?,
            ));

            // Ensure storage root is a container.
            if !stroot_context.is_left_classified() {
                return Err(INVALID_STORAGE_ROOT_URI.new_problem_builder().finish());
            }

            // Get storage root's assoc base object's metadata.
            let stroot_base_object = stroot_context
                .assoc_odr_object_map()
                .base_object()
                .as_left_classified()
                .expect("Container's assoc base object must be a dir object.");

            let stroot_base_obj_is_exists = stroot_base_object.is_exist().await.map_err(|e| {
                error!(
                    "Unknown io error in checking existence of storage root's assoc base object."
                );
                UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
            })?;

            // Create the storage root base object, if doesn't exist.
            if !stroot_base_obj_is_exists {
                if !(backend_caps.create_dir) {
                    error!(
                "Backend doesn't have create_dir capabilities to init non existing storage root."
            );
                    return Err(UNSUPPORTED_OPERATION.new_problem());
                }

                // Create storage root's base object.
                stroot_base_object.create().await.map_err(|e| {
                    error!("Unknown io error in creating storage root's assoc base object.");
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })?;
            }

            // Repo considered initialized.
            Ok(true)
        })
    }
}

impl<Setup: ODRSetup> RepoInitializer for ODRInitializer<Setup> {}
