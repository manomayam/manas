//! I define an implementation of [`BaseMethodService`](super::super::BaseMethodService)
//! for handling `DELETE` method over solid resources.
//!

use std::{sync::Arc, task::Poll};

use dyn_problem::{type_::INTERNAL_ERROR, Problem, ProblemBuilderExt};
use futures::{future::BoxFuture, TryFutureExt};
use http::{Method, Request, StatusCode};
use http_api_problem::ApiError;
use hyper::Body;
use manas_access_control::model::{KResolvedAccessControl, KResolvedHostAccessControl};
use manas_http::uri::invariant::NormalAbsoluteHttpUri;
use manas_repo::{
    policy::uri::RepoUriPolicy,
    service::resource_operator::{
        common::{
            preconditions::{impl_::http::HttpPreconditions, KEvaluatedRepValidators},
            problem::{ACCESS_DENIED, PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_OPERATION},
            status_token::ExistingRepresentedResourceToken,
        },
        deleter::{
            ResourceDeleteRequest, ResourceDeleteResponse, ResourceDeleteTokenSet,
            DELETE_TARGETS_NON_EMPTY_CONTAINER, DELETE_TARGETS_STORAGE_ROOT,
        },
    },
    RepoExt,
};
use manas_space::resource::uri::SolidResourceUri;
use manas_specs::{
    protocol::{
        REQ_SERVER_DELETE_PROTECT_NONEMPTY_CONTAINER, REQ_SERVER_DELETE_PROTECT_ROOT_CONTAINER,
    },
    SpecProblem,
};
use name_locker::{LockKind, NameLocker};
use tower::{Service, ServiceExt};
use tracing::error;
use typed_record::{ClonableTypedRecord, TypedRecord};

use crate::{
    service::method::common::snippet::{
        op_req::KOpReqExtensions, req_headers::etag_base_normalized_conditional_headers,
        status_token::resolve_status_token,
    },
    SgCredentials, SgRepo, SgResourceDeleter, SgResourceStatusToken, SolidStorage,
};

/// A service that handles conditional `DELETE` request over a resource in solid compatible, concurrent safe way.
#[derive(Debug)]
pub struct BaseDeleteService<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,
}

impl<Storage> Clone for BaseDeleteService<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

/// Type of future returned by [`BaseDeleteService`].
pub type BaseDeleteResponseFuture<Storage> =
    BoxFuture<'static, Result<ResourceDeleteResponse<SgRepo<Storage>>, ApiError>>;

impl<Storage> Service<Request<Body>> for BaseDeleteService<Storage>
where
    Storage: SolidStorage,
{
    type Response = ResourceDeleteResponse<Storage::Repo>;

    type Error = ApiError;

    type Future = BaseDeleteResponseFuture<Storage>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Will be always ready.
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "BaseDeleteService::call")]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(Self::apply(self.storage.clone(), req))
    }
}

impl<Storage> BaseDeleteService<Storage>
where
    Storage: SolidStorage,
{
    /// Create a new [`BaseDeleteService`].
    #[inline]
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Apply DELETE method.
    async fn apply(
        storage: Arc<Storage>,
        mut req: Request<Body>,
    ) -> Result<ResourceDeleteResponse<Storage::Repo>, ApiError> {
        // Ensure method is DELETE
        if req.method() != Method::DELETE {
            panic!("BaseDeleteService must be routed to for only DELETE requests");
        }

        // Get normal resource uri.
        let res_uri = req
            .extensions()
            .get::<NormalAbsoluteHttpUri>()
            .expect("BaseDeleteService must be called after confirming resource uri is normal absolute http uri.").clone();

        // Uri policy of the repo.
        let uri_policy = storage.repo().uri_policy();

        // Get res lock name.
        let res_lock_name = uri_policy.mutex_normal_res_uri_hash(&res_uri);

        // Construct delete res future.
        let res_delete_fut = Box::pin(Self::conditional_delete(
            storage.clone(),
            res_uri.clone(),
            HttpPreconditions {
                method: Method::DELETE,
                preconditions: etag_base_normalized_conditional_headers(req.headers()),
            },
            req.extensions_mut()
                .remove::<SgCredentials<Storage>>()
                .unwrap_or_default(),
            req.extensions_mut()
                .remove_rec_item::<KOpReqExtensions>()
                .unwrap_or_default(),
        )) as BaseDeleteResponseFuture<Storage>;

        // Perform delete with exclusive lock on request target.
        storage
            .resource_locker()
            .poll_with_lock(res_delete_fut, Some(res_lock_name), LockKind::Exclusive)
            .await
    }

    /// Delete a resource conditionally.
    #[tracing::instrument(
        skip_all,
        name = "BaseDeleteService:conditional_delete",
        fields(res_uri, preconditions,)
    )]
    async fn conditional_delete(
        storage: Arc<Storage>,
        res_uri: SolidResourceUri,
        preconditions: HttpPreconditions,
        credentials: SgCredentials<Storage>,
        op_req_extensions: ClonableTypedRecord,
    ) -> Result<ResourceDeleteResponse<Storage::Repo>, ApiError> {
        // Resolve resource status token.
        let status_token: SgResourceStatusToken<Storage> =
            resolve_status_token(storage.as_ref(), res_uri.clone()).await?;

        // Ensure resource is represented.
        let er_token = status_token.existing_represented().ok_or_else(|| {
            error!("Resource is not existing represented.");
            ApiError::builder(StatusCode::NOT_FOUND)
        })?;

        // Call the deleter.
        SgResourceDeleter::<Storage>::default()
            .ready()
            .and_then(|svc| {
                let res_slot = er_token.slot().clone();
                let mut fut = svc.call(ResourceDeleteRequest {
                    tokens: ResourceDeleteTokenSet::new(er_token),
                    preconditions: Box::new(preconditions),
                    credentials,
                    extensions: op_req_extensions,
                });
                // If resource has a host resource, then
                // proceed with exclusive lock on it.
                if let Some(slot_rev_link) = res_slot.slot_rev_link() {
                    let host_res_lock_name = storage
                        .repo()
                        .uri_policy()
                        .mutex_normal_res_uri_hash(&slot_rev_link.target);

                    fut = storage.resource_locker().poll_with_lock(
                        fut,
                        Some(host_res_lock_name),
                        LockKind::Exclusive,
                    )
                }

                fut
            })
            .await
            .map_err(Self::map_problem)
    }

    /// Map internal problem to api error.
    fn map_problem(problem: Problem) -> ApiError {
        if ACCESS_DENIED.is_type_of(&problem) {
            error!("Access denied.");
            ApiError::builder(StatusCode::FORBIDDEN).message("Not authorized.")
        }
        // If pre conditions not satisfied.
        else if PRECONDITIONS_NOT_SATISFIED.is_type_of(&problem) {
            ApiError::builder(StatusCode::PRECONDITION_FAILED)
                .extend_with_opt::<KEvaluatedRepValidators>(
                    problem
                        .extensions()
                        .get_rv::<KEvaluatedRepValidators>()
                        .cloned(),
                )
        }
        // If target resource is  storage root or it's acl,
        // Req: When a DELETE request targets storage’s root
        // container or its associated ACL resource, the
        // server MUST respond with the 405 status code.
        else if DELETE_TARGETS_STORAGE_ROOT.is_type_of(&problem) {
            SpecProblem::new(StatusCode::METHOD_NOT_ALLOWED)
                .with_recourse_as_per(&REQ_SERVER_DELETE_PROTECT_ROOT_CONTAINER)
                .into()
        }
        // If target resource is a non empty container.
        // Req: When a DELETE request targets a container,
        // the server MUST delete the container if it
        // contains no resources. If the container contains
        // resources, the server MUST respond with the 409
        // status code and response body describing the error.
        else if DELETE_TARGETS_NON_EMPTY_CONTAINER.is_type_of(&problem) {
            SpecProblem::new(StatusCode::CONFLICT)
                .with_recourse_as_per(&REQ_SERVER_DELETE_PROTECT_NONEMPTY_CONTAINER)
                .into()
        }
        // If operation is not supported.
        else if UNSUPPORTED_OPERATION.is_type_of(&problem) {
            error!("Unsupported operation.");
            ApiError::builder(StatusCode::METHOD_NOT_ALLOWED)
        } else {
            INTERNAL_ERROR.new_api_error_builder(StatusCode::INTERNAL_SERVER_ERROR)
        }
        .extend_with_opt::<KResolvedAccessControl<SgCredentials<Storage>>>(
            problem
                .extensions()
                .get_rv::<KResolvedAccessControl<SgCredentials<Storage>>>()
                .cloned(),
        )
        .extend_with_opt::<KResolvedHostAccessControl<SgCredentials<Storage>>>(
            problem
                .extensions()
                .get_rv::<KResolvedHostAccessControl<SgCredentials<Storage>>>()
                .cloned(),
        )
        .finish()
    }
}
