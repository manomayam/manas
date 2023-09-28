//! I define an implementation of [`BaseMethodService`](super::super::BaseMethodService)
//! for handling `POST` method over solid resources.
//!

use std::{sync::Arc, task::Poll};

use dyn_problem::{type_::INTERNAL_ERROR, Problem, ProblemBuilderExt};
use futures::{future::BoxFuture, TryFutureExt};
use headers::HeaderMapExt;
use http::{Method, Request, StatusCode};
use http_api_problem::ApiError;
use hyper::Body;
use manas_access_control::model::{KResolvedAccessControl, KResolvedHostAccessControl};
use manas_http::{
    header::{
        link::{Link, LinkValue, TYPE_REL_TYPE},
        slug::Slug,
    },
    representation::{
        impl_::{basic::BasicRepresentation, common::data::bytes_stream::BytesStream},
        metadata::{KContentType, RepresentationMetadata},
    },
};
use manas_repo::{
    policy::uri::RepoUriPolicy,
    service::resource_operator::{
        common::{
            preconditions::{impl_::http::HttpPreconditions, KEvaluatedRepValidators},
            problem::{
                ACCESS_DENIED, INVALID_RDF_SOURCE_REPRESENTATION,
                INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA,
                INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES, PAYLOAD_TOO_LARGE,
                PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_MEDIA_TYPE, UNSUPPORTED_OPERATION,
                URI_POLICY_VIOLATION,
            },
            rep_update_action::RepUpdateAction,
            status_token::ExistingRepresentedResourceToken,
        },
        creator::{ResourceCreateRequest, ResourceCreateResponse, ResourceCreateTokenSet},
    },
    RepoExt,
};
use manas_space::resource::{
    kind::SolidResourceKind, slot_rel_type::SlotRelationType, uri::SolidResourceUri,
};
use manas_specs::{
    protocol::{REQ_SERVER_POST_TARGET_NOT_FOUND, REQ_SERVER_PROTECT_CONTAINED_RESOURCE_METADATA},
    SpecProblem,
};
use name_locker::{LockKind, NameLocker};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rdf_utils::model::term::CompatTerm;
use rdf_vocabularies::ns;
use sophia_api::term::Term;
use tower::{Service, ServiceExt};
use tracing::{debug, error, info};
use typed_record::{ClonableTypedRecord, TypedRecord};

use crate::{
    service::method::common::snippet::{
        op_req::KOpReqExtensions,
        req_headers::{
            etag_base_normalized_conditional_headers, resolve_preconditions_eval_status,
            resolve_req_content_length_hint, resolve_req_content_type,
        },
        status_token::resolve_status_token,
    },
    SgCredentials, SgRepo, SgResourceCreator, SgResourceStatusToken, SolidStorage,
};

/// Type of response of the `BasePostService`.
pub type BasePostResponse<Storage> = ResourceCreateResponse<SgRepo<Storage>>;

/// A service that handles conditional `POST` request over a
/// resource in solid compatible, concurrent safe way.
#[derive(Debug)]
pub struct BasePostService<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,
}

impl<Storage> Clone for BasePostService<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

/// Type of future returned by [`BasePostService`]..
pub type BasePostResponseFuture<Storage> =
    BoxFuture<'static, Result<BasePostResponse<Storage>, ApiError>>;

impl<Storage> Service<Request<Body>> for BasePostService<Storage>
where
    Storage: SolidStorage,
{
    type Response = BasePostResponse<Storage>;

    type Error = ApiError;

    type Future = BasePostResponseFuture<Storage>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Will be always ready.
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "BasePutService::call")]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(Self::apply(self.storage.clone(), req))
    }
}

impl<Storage> BasePostService<Storage>
where
    Storage: SolidStorage,
{
    /// Create a new [`BasePostService`].
    #[inline]
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Apply Post method.
    #[tracing::instrument(skip_all)]
    async fn apply(
        storage: Arc<Storage>,
        req: Request<Body>,
    ) -> Result<BasePostResponse<Storage>, ApiError> {
        // Ensure method is POST
        if req.method() != Method::POST {
            panic!("BasePostService must be routed to for only POST requests");
        }

        let (mut req_parts, body) = req.into_parts();

        // Assume request target is a container.
        // If it is resolved to be not, we will through error
        // later.

        // Get normal resource uri.
        let container_uri = req_parts
            .extensions
            .get::<SolidResourceUri>()
            .expect("BasePostService must be called after confirming resource uri is normal absolute http uri.").clone();

        let container_preconditions = HttpPreconditions {
            method: Method::POST,
            preconditions: etag_base_normalized_conditional_headers(&req_parts.headers),
        };

        // Get links.
        let links = req_parts.headers.typed_get::<Link>().unwrap_or_default();

        // Get interaction mode.
        let new_res_kind = if links.values.iter().any(|link: &LinkValue| {
            link.rel().rel_types.contains(&*TYPE_REL_TYPE)
                && Term::eq(
                    &CompatTerm::from_uri_ref(&link.target().0),
                    ns::ldp::BasicContainer,
                )
        }) {
            // Req: Servers MUST create a container with URI
            // path ending /{id}/ in container / for requests
            // including the HTTP Link header with rel="type"
            // targeting a valid LDP container type.
            SolidResourceKind::Container
        } else {
            SolidResourceKind::NonContainer
        };

        info!(
            "Resolved kind of the resource to be created: {:?}",
            new_res_kind
        );

        // Uri policy of the repo.
        let uri_policy = storage.repo().uri_policy();

        // Get provided slug, or create a random one.
        // Req: When a successful POST request creates a
        // resource, the server MUST assign a URI to that
        // resource.
        // Servers MAY allow clients to suggest the URI of a
        // resource created through POST, using the HTTP Slug
        // header as defined in [RFC5023].
        let slug = req_parts.headers.typed_get::<Slug>().unwrap_or_else(|| {
            // Generate random string
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect::<String>()
                .into()
        });

        debug!("Resolved slug: {}", slug);

        // Resolve uri for new resource.
        let new_res_uri: SolidResourceUri = uri_policy
            .suggest_res_uri(&container_uri, &slug, new_res_kind)
            .map_err(|_| {
                ApiError::builder(StatusCode::BAD_REQUEST)
                    .message("Given slug is unfulfillable")
                    .finish()
            })?;

        info!(
            "Resolved uri for the resource to be created: {:?}",
            new_res_uri
        );
        // Get container lock name.
        let res_lock_name = uri_policy.mutex_normal_res_uri_hash(&container_uri);
        // Get new res lock name.
        let new_res_lock_name = uri_policy.mutex_normal_res_uri_hash(&new_res_uri);

        let new_res_rep_action = RepUpdateAction::SetWith(
            BasicRepresentation {
                metadata: RepresentationMetadata::new()
                    .with::<KContentType>(resolve_req_content_type(&req_parts.headers)?),
                data: BytesStream::from_hyper_body(
                    body,
                    Some(resolve_req_content_length_hint(&req_parts.headers)),
                ),
                base_uri: Some(new_res_uri.clone().into_subject()),
            }
            .into_binary(),
        );

        // Construct create future.
        let new_res_create_fut = Self::conditional_create(
            storage.clone(),
            container_uri.clone(),
            new_res_uri,
            new_res_kind,
            new_res_rep_action,
            container_preconditions,
            req_parts
                .extensions
                .remove::<SgCredentials<Storage>>()
                .unwrap_or_default(),
            req_parts
                .extensions
                .remove_rec_item::<KOpReqExtensions>()
                .unwrap_or_default(),
        );

        // Create new resource with locks over itself and container.
        // Locks must be in host up order to avoid dead-locks.
        storage
            .resource_locker()
            // With exclusive lock on container request target.
            .poll_with_lock(
                // With exclusive lock on resource to be created.
                storage.resource_locker().poll_with_lock(
                    new_res_create_fut,
                    Some(new_res_lock_name),
                    LockKind::Exclusive,
                ),
                Some(res_lock_name),
                LockKind::Exclusive,
            )
            .await
    }

    /// Create a new contained resource conditionally.
    #[tracing::instrument(
        skip_all,
        name = "BasePostService::conditional_create",
        fields(container_uri, new_res_uri, new_res_kind, container_preconditions)
    )]
    #[allow(clippy::too_many_arguments)]
    async fn conditional_create(
        storage: Arc<Storage>,
        container_uri: SolidResourceUri,
        new_res_uri: SolidResourceUri,
        new_res_kind: SolidResourceKind,
        new_res_rep_action: RepUpdateAction<Storage::Repo>,
        container_preconditions: HttpPreconditions,
        credentials: SgCredentials<Storage>,
        op_req_extensions: ClonableTypedRecord,
    ) -> Result<BasePostResponse<Storage>, ApiError> {
        // Resolve container status token.
        let container_status_token: SgResourceStatusToken<Storage> =
            resolve_status_token(storage.as_ref(), container_uri.clone()).await?;

        // Ensure container is represented.
        let container_er_token =
            container_status_token
                .existing_represented()
                .ok_or_else(|| {
                    error!("Container is not existing represented.");
                    // Req: When a POST method request targets a resource
                    // without an existing representation, the server
                    // MUST respond with the 404 status code.
                    ApiError::from(
                        SpecProblem::new(StatusCode::NOT_FOUND)
                            .with_recourse_as_per(&REQ_SERVER_POST_TARGET_NOT_FOUND),
                    )
                })?;

        // Ensure request target is indeed a container as
        // assumed.
        if !container_er_token.slot().is_container_slot() {
            error!("POST targets a non container.");
            return Err(ApiError::builder(StatusCode::NOT_IMPLEMENTED)
                .message("POST on non container resources is not implemented.")
                .finish());
        }

        // Resolve new resource status token.
        let new_res_status_token: SgResourceStatusToken<Storage> =
            resolve_status_token(storage.as_ref(), new_res_uri.clone()).await?;

        // Resolve conflict free token
        let new_res_cf_token = new_res_status_token
            .non_existing_mutex_non_existing()
            .ok_or_else(|| {
                error!("Slug is unfulfillable.");
                // TODO Should try with another random slug.
                ApiError::builder(StatusCode::BAD_REQUEST)
                    .message("Given slug is unfulfillable")
                    .finish()
            })?;

        // Construct the request.
        let new_res_create_request = ResourceCreateRequest::<Storage::Repo> {
            tokens: ResourceCreateTokenSet::try_new(new_res_cf_token, container_er_token)
                .expect("Must be repo consistent tokens."),
            resource_kind: new_res_kind,
            slot_rev_rel_type: SlotRelationType::Contains,
            rep_update_action: new_res_rep_action,
            host_preconditions: Box::new(container_preconditions),
            credentials,
            extensions: op_req_extensions,
        };

        // Call the creator.
        SgResourceCreator::<Storage>::default()
            .ready()
            .and_then(|svc| svc.call(new_res_create_request))
            .await
            .map_err(Self::map_problem)
    }

    /// Map internal problem to api error.
    fn map_problem(problem: Problem) -> ApiError {
        // If access is denied.
        if ACCESS_DENIED.is_type_of(&problem) {
            error!("Access denied.");
            ApiError::builder(StatusCode::FORBIDDEN).message("Not authorized.")
        }
        // If preconditions not satisfied.
        else if PRECONDITIONS_NOT_SATISFIED.is_type_of(&problem) {
            ApiError::builder(   resolve_preconditions_eval_status(&problem).unwrap_or(StatusCode::PRECONDITION_FAILED))
            .extend_with_opt::<KEvaluatedRepValidators>(problem.extensions().
            // TODO evaluated "host" rep validators?
            get_rv::<KEvaluatedRepValidators>().cloned())
        }
        else if URI_POLICY_VIOLATION.is_type_of(&problem)
        {
            ApiError::builder(StatusCode::BAD_REQUEST)
                .message("Given slug is unfulfillable")
        }
        // If rep media type is not supported.
        else if UNSUPPORTED_MEDIA_TYPE.is_type_of(&problem) {
            ApiError::builder(StatusCode::UNSUPPORTED_MEDIA_TYPE).message("Unsupported representation content-type.")
        }
        // If new resource is to be a container, and posted 
        // rep is not a valid rdf doc.
        else if INVALID_RDF_SOURCE_REPRESENTATION.is_type_of(&problem) {
            ApiError::builder(StatusCode::BAD_REQUEST)
                .message("Invalid Rdf source representation.")
        }
        // If user supplied container rep has containment 
        // triples,
        else if INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES.is_type_of(&problem) {
            ApiError::builder(StatusCode::CONFLICT).message("Supplied container representation contains containment statements in it. That is not supported")
        }
        // If user supplied container rep has contained res metadata.
        // Req: Servers MUST NOT allow HTTP POST, PUT and 
        // PATCH to update a container’s resource metadata 
        // statements; if the server receives such a request, 
        // it MUST respond with a 409 status code.
        else if INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA.is_type_of(&problem) {
            SpecProblem::new(StatusCode::CONFLICT)
                .with_recourse_as_per(&REQ_SERVER_PROTECT_CONTAINED_RESOURCE_METADATA)
                .into()
        }
        // If payload is too large.
        else if PAYLOAD_TOO_LARGE.is_type_of(&problem) {
            ApiError::builder(StatusCode::PAYLOAD_TOO_LARGE)
        }
        // If operation is not supported.
        else if UNSUPPORTED_OPERATION.is_type_of(&problem) {
            error!("Unsupported operation.");
            ApiError::builder(StatusCode::METHOD_NOT_ALLOWED)
        }
        // On any other error.
        else {
            INTERNAL_ERROR.new_api_error_builder(StatusCode::INTERNAL_SERVER_ERROR)
        }
        .extend_with_opt::<KResolvedAccessControl<SgCredentials<Storage>>>(
            problem
                .extensions()
                // NOTE: resolved "host" access control.
                .get_rv::<KResolvedHostAccessControl<SgCredentials<Storage>>>()
                .cloned(),
        )
        .finish()
    }
}
