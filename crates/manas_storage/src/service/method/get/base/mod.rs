//! I define an implementation of [`BaseMethodService`](super::super::BaseMethodService)
//! for handling `GET` method over solid resources.
//!

use std::{
    sync::Arc,
    task::{Context, Poll},
};

use dyn_problem::{Problem, ProblemBuilderExt};
use futures::{future::BoxFuture, TryFutureExt};
use headers::{HeaderMap, HeaderMapExt};
use http::{request::Parts, Method, Request, StatusCode};
use http_api_problem::ApiError;
use manas_access_control::model::KResolvedAccessControl;
use manas_http::body::Body;
use manas_http::{
    conditional_req::PreconditionsResolvedAction,
    representation::impl_::{
        basic::BasicRepresentation, binary::BinaryRepresentation,
        common::data::bytes_stream::BytesStream,
    },
    uri::invariant::NormalAbsoluteHttpUri,
};
use manas_repo::{
    policy::uri::RepoUriPolicy,
    service::resource_operator::{
        common::{
            preconditions::{impl_::http::HttpPreconditions, KEvaluatedRepValidators},
            problem::{ACCESS_DENIED, PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_OPERATION},
            status_token::{ExistingResourceToken, ResourceStatusToken},
        },
        reader::{
            rep_preferences::{
                range_negotiator::impl_::ConditionalRangeNegotiator,
                ContainerRepresentationPreference, RepresentationPreferences,
            },
            ConnegParams, ResourceReadRequest, ResourceReadResponse, ResourceReadTokenSet,
            RANGE_NOT_SATISFIABLE,
        },
    },
    RepoExt,
};
use name_locker::{LockKind, NameLocker};
use tower::{Service, ServiceExt};
use tracing::{debug, error, info};
use typed_record::TypedRecord;

use crate::{
    service::method::{
        common::snippet::{
            op_req::KOpReqExtensions,
            req_headers::{
                etag_base_normalized_conditional_headers, resolve_preconditions_eval_status,
            },
            status_token::resolve_status_token,
        },
        get::base::error_context::KExistingMutexResourceUri,
    },
    SgCredentials, SgRepo, SgResourceReader, SgResourceStatusToken, SolidStorage,
};

pub mod error_context;

/// A service that handles conditional GET request over a
/// solid resource by resolving it's metadata and
/// selected-representation in concurrent safe way.
///
/// This service can be used as inner service for
/// further middleware to layer over conditional resolution
/// of selected representation and resource metadata.
#[derive(Debug)]
pub struct BaseGetService<Storage>
where
    Storage: SolidStorage,
{
    /// Storage.
    pub storage: Arc<Storage>,
}

impl<Storage> Clone for BaseGetService<Storage>
where
    Storage: SolidStorage,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

/// Type alias for response type of [`BaseGetService`].
pub type BaseGetResponse<Storage> = ResourceReadResponse<SgRepo<Storage>, BinaryRepresentation>;

/// Type of future returned by [`BaseGetService`].
pub type BaseGetServiceFuture<Storage> =
    BoxFuture<'static, Result<BaseGetResponse<Storage>, ApiError>>;

impl<Storage> Service<Request<Body>> for BaseGetService<Storage>
where
    Storage: SolidStorage,
{
    type Response = BaseGetResponse<Storage>;

    type Error = ApiError;

    type Future = BaseGetServiceFuture<Storage>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Will be always ready.
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "BaseGetService::call")]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(Self::apply(self.storage.clone(), req))
    }
}

impl<Storage> BaseGetService<Storage>
where
    Storage: SolidStorage,
{
    /// Create a new [`BaseGetService`].
    #[inline]
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Apply the Get / Head method.
    #[tracing::instrument(skip_all)]
    async fn apply(
        storage: Arc<Storage>,
        req: Request<Body>,
    ) -> Result<BaseGetResponse<Storage>, ApiError> {
        // Ensure method is GET / HEAD.
        if ![Method::GET, Method::HEAD].contains(req.method()) {
            panic!("BaseGet service must be routed to for only GET or HEAD requests");
        }

        // Get normal resource uri.
        let res_uri = req
            .extensions()
            .get::<NormalAbsoluteHttpUri>()
            .expect("BaseGetService must be called after ensuring resource uri is normal absolute http uri.").clone();

        // Get res lock name.
        let res_lock_name = storage
            .repo()
            .uri_policy()
            .mutex_normal_res_uri_hash(&res_uri);

        let (req_parts, _) = req.into_parts();

        // Resolve response with shared lock over resource.
        storage
            .resource_locker()
            .poll_with_lock(
                Self::conditional_get(storage.clone(), res_uri, res_lock_name.clone(), req_parts),
                Some(res_lock_name.clone()),
                LockKind::Shared,
            )
            .await
    }

    /// Map the inner problem to api error
    fn map_problem(problem: Problem) -> ApiError {
        if ACCESS_DENIED.is_type_of(&problem) {
            error!("Access denied.");
            ApiError::builder(StatusCode::FORBIDDEN).message("Not authorized.")
        } else if PRECONDITIONS_NOT_SATISFIED.is_type_of(&problem) {
            error!("Pre conditions not satisfied.");
            ApiError::builder(
                resolve_preconditions_eval_status(&problem).unwrap_or(StatusCode::NOT_MODIFIED),
            )
            .extend_with_opt::<KEvaluatedRepValidators>(
                problem
                    .extensions()
                    .get_rv::<KEvaluatedRepValidators>()
                    .cloned(),
            )
        } else if RANGE_NOT_SATISFIABLE.is_type_of(&problem) {
            error!("Range not satisfiable.");
            ApiError::builder(StatusCode::RANGE_NOT_SATISFIABLE)
        } else if UNSUPPORTED_OPERATION.is_type_of(&problem) {
            error!("Unsupported operation.");
            ApiError::builder(StatusCode::METHOD_NOT_ALLOWED)
        } else {
            error!(
                "Unknown error in getting resource state. Error:\n {:?}",
                problem
            );
            ApiError::builder(StatusCode::INTERNAL_SERVER_ERROR)
        }
        .extend_with_opt::<KResolvedAccessControl<SgCredentials<Storage>>>(
            problem
                .extensions()
                .get_rv::<KResolvedAccessControl<SgCredentials<Storage>>>()
                .cloned(),
        )
        .finish()
    }

    /// Get resource conditionally.
    #[tracing::instrument(skip_all, name = "BaseGetService::conditional_get")]
    async fn conditional_get(
        storage: Arc<Storage>,
        res_uri: NormalAbsoluteHttpUri,
        rep_stream_lock_name: String,
        mut req_parts: Parts,
    ) -> Result<BaseGetResponse<Storage>, ApiError> {
        // Resolve etag-base-normalized conditional headers.
        let base_normal_conditional_headers =
            etag_base_normalized_conditional_headers(&req_parts.headers);

        // Resolve status token.
        let status_token: SgResourceStatusToken<Storage> =
            resolve_status_token(storage.as_ref(), res_uri.clone()).await?;

        // Resolve status token as existing-represented
        // status token.
        let er_token = match status_token {
            ResourceStatusToken::Existing(e_token) => match e_token {
                ExistingResourceToken::NonRepresented(_en_token) => {
                    error!("Resource is not represented.");
                    Err(ApiError::builder(StatusCode::NOT_FOUND).finish())
                }
                ExistingResourceToken::Represented(token) => {
                    info!("Resource exists and is represented.");
                    Ok(token)
                }
            },
            ResourceStatusToken::NonExisting(ne_token) => {
                error!("Resource doesn't exists.");
                Err(ApiError::builder(if ne_token.was_existing() {
                    debug!("Resource was existing once.");
                    StatusCode::GONE
                } else {
                    StatusCode::NOT_FOUND
                })
                // Attach any mutex context.
                .extend_with_opt::<KExistingMutexResourceUri>(
                    ne_token
                        .existing_mutex_slot()
                        .map(|slot| slot.id().uri.clone()),
                )
                .finish())
            }
        }?;

        // Construct resource state request.
        let res_state_request = ResourceReadRequest {
            tokens: ResourceReadTokenSet::new(er_token),
            rep_preferences: RepresentationPreferences {
                // For containers, request full representation.
                // TODO should honour `Prefer` header.
                container_rep_preference: if req_parts.method == Method::HEAD {
                    ContainerRepresentationPreference::Minimal
                } else {
                    ContainerRepresentationPreference::All
                },

                // For non-containers, use conditional range negotiator.
                non_container_rep_range_negotiator: Box::new(ConditionalRangeNegotiator {
                    range: req_parts.headers.typed_get(),
                    if_range: base_normal_conditional_headers.typed_get(),
                }),
            },
            rep_conneg_params: ConnegParams {
                accept: req_parts.headers.typed_get(),
            },
            preconditions: Box::new(HttpPreconditions {
                method: req_parts.method.clone(),
                // Preconditions will be evaluated post conneg
                // to account for browser behavior with conneg and etags.
                preconditions: HeaderMap::new(),
            }),
            credentials: req_parts
                .extensions
                .remove::<SgCredentials<Storage>>()
                .unwrap_or_default(),
            extensions: req_parts
                .extensions
                .remove_rec_item::<KOpReqExtensions>()
                .unwrap_or_default(),
        };

        let mut reader = SgResourceReader::<Storage>::default();

        // Query resource state.
        let resp = reader
            .ready()
            .and_then(|svc| svc.call(res_state_request))
            .inspect_err(|e| error!("Error in resolving resource state. Error:\n {}", e))
            .await
            .map_err(Self::map_problem)?;

        // Evaluate pre conditions for represented resource.
        let pc_resolved_action = HttpPreconditions {
            method: req_parts.method,
            // NOTE: headers with out etag-normalization.
            preconditions: req_parts.headers,
        }
        .evaluate_raw(Some(resp.state.representation_metadata()))
        .0;

        // If preconditions not satisfied, then return
        if let PreconditionsResolvedAction::Return(status) = pc_resolved_action {
            error!("Preconditions not satisfied");
            return Err(ApiError::builder(status)
                .extend_with::<KEvaluatedRepValidators>(Some(
                    resp.state.into_parts().1.into_basic().metadata,
                ))
                .finish());
        };

        debug!("Preconditions satisfied");

        // Lock resource name throughout rep content stream lifetime, with shared lock.
        // NOTE: must set timeout
        // TODO should provide map_data method.
        let resp = resp.map_representation(move |rep| {
            let rep = rep.into_streaming().into_basic();
            BasicRepresentation {
                metadata: rep.metadata,
                base_uri: rep.base_uri,
                data: BytesStream {
                    stream: storage.resource_locker().poll_read_with_lock(
                        rep.data.stream,
                        Some(rep_stream_lock_name),
                        LockKind::Shared,
                    ),
                    size_hint: rep.data.size_hint,
                },
            }
            .into_binary()
        });

        // Return resource state response.
        Ok(resp)
    }
}
