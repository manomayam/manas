//! I define an implementation of [`BaseMethodService`](super::super::BaseMethodService)
//! for handling `PUT`/`PATCH` method over solid resources.
//!

use std::{error::Error, sync::Arc, task::Poll};

use dyn_problem::{type_::INTERNAL_ERROR, Problem, ProblemBuilderExt};
use futures::{future::BoxFuture, TryFutureExt};
use http::{Method, Request, StatusCode};
use http_api_problem::{ApiError, ApiErrorBuilder};
use http_body::SizeHint;
use manas_access_control::model::{KResolvedAccessControl, KResolvedHostAccessControl};
use manas_http::{
    body::Body,
    conditional_req::PreconditionsResolvedAction,
    header::common::media_type::{MediaType, TEXT_TURTLE},
    representation::{
        impl_::{basic::BasicRepresentation, common::data::bytes_stream::BytesStream},
        metadata::{KContentType, RepresentationMetadata},
    },
    uri::invariant::NormalAbsoluteHttpUri,
};
use manas_repo::{
    policy::uri::RepoUriPolicy,
    service::{
        patcher_resolver::{INVALID_ENCODED_PATCH, UNKNOWN_PATCH_DOC_CONTENT_TYPE},
        resource_operator::{
            common::{
                preconditions::{impl_::http::HttpPreconditions, KEvaluatedRepValidators},
                problem::{
                    ACCESS_DENIED, INVALID_RDF_SOURCE_REPRESENTATION,
                    INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA,
                    INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES, PAYLOAD_TOO_LARGE,
                    PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_MEDIA_TYPE, UNSUPPORTED_OPERATION,
                    URI_POLICY_VIOLATION,
                },
                rep_patcher::{
                    INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE, INVALID_ENCODED_SOURCE_REP,
                    PATCH_SEMANTICS_ERROR,
                },
                rep_update_action::RepUpdateAction,
                status_token::{
                    ExistingRepresentedResourceToken, NonExistingMutexNonExistingResourceToken,
                    NonExistingResourceToken, ResourceStatusToken,
                },
            },
            creator::{ResourceCreateRequest, ResourceCreateResponse, ResourceCreateTokenSet},
            updater::{ResourceUpdateRequest, ResourceUpdateResponse, ResourceUpdateTokenSet},
        },
    },
    Repo, RepoExistingResourceToken, RepoExt,
};
use manas_space::{
    resource::{
        kind::SolidResourceKind, slot::SolidResourceSlot, slot_id::SolidResourceSlotId,
        slot_path::invariant::SansAuxLinkRelativeResourceSlotPath, slot_rel_type::SlotRelationType,
        uri::SolidResourceUri,
    },
    RelativeSolidStorageSpace,
};
use manas_specs::{
    protocol::{
        REQ_SERVER_PATCH_N3_INVALID, REQ_SERVER_PROTECT_CONTAINED_RESOURCE_METADATA,
        REQ_SERVER_PROTECT_CONTAINMENT, REQ_SERVER_URI_TRAILING_SLASH_DISTINCT,
    },
    SpecProblem,
};
use name_locker::{LockKind, NameLocker};
use tower::{Service, ServiceExt};
use tracing::{error, info, warn};
use typed_record::{ClonableTypedRecord, TypedRecord, TypedRecordKey};

use super::marshaller::default::KPatchErrorContext;
use crate::{
    service::method::common::snippet::{
        op_req::KOpReqExtensions,
        req_headers::{
            etag_base_normalized_conditional_headers, resolve_preconditions_eval_status,
            resolve_req_content_length_hint, resolve_req_content_type,
        },
        status_token::resolve_status_token,
    },
    SgCredentials, SgRepPatcher, SgResourceConflictFreeToken, SgResourceCreator,
    SgResourceStatusToken, SgResourceUpdater, SolidStorage, SolidStorageExt,
};

/// Response of the `BasePutOrPatchService`.
#[derive(Debug, Clone)]
pub struct BasePutOrPatchResponse<Storage: SolidStorage> {
    /// Slot of the upserted resource.
    pub upserted_res_slot: SolidResourceSlot<Storage::StSpace>,

    /// If resource is newly created.
    // TODO can return list of created resources.
    pub is_created: bool,

    /// Validators for new representation, if info is available.
    pub new_rep_validators: Option<RepresentationMetadata>,
}

/// A service that handles conditional `PUT` / `PATCH`
/// request over a resource in solid compatible, concurrent
/// safe way.
#[derive(Debug)]
pub struct BasePutOrPatchService<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,
}

impl<Storage> Clone for BasePutOrPatchService<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

/// Type of future returned by [`BasePutOrPatchService`].
pub type BasePutOrPatchResponseFuture<Storage> =
    BoxFuture<'static, Result<BasePutOrPatchResponse<Storage>, ApiError>>;

impl<Storage> Service<Request<Body>> for BasePutOrPatchService<Storage>
where
    Storage: SolidStorage,
{
    type Response = BasePutOrPatchResponse<Storage>;

    type Error = ApiError;

    type Future = BasePutOrPatchResponseFuture<Storage>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Will be always ready.
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "BasePutOrPatchService::call")]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(Self::apply(self.storage.clone(), req))
    }
}

impl<Storage> BasePutOrPatchService<Storage>
where
    Storage: SolidStorage,
{
    /// Create a new [`BasePutOrPatchService`].
    #[inline]
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Apply PutOrPatch method.
    #[tracing::instrument(skip_all)]
    async fn apply(
        storage: Arc<Storage>,
        req: Request<Body>,
    ) -> Result<BasePutOrPatchResponse<Storage>, ApiError> {
        // Ensure method is PUT / PATCH
        if ![Method::PUT, Method::PATCH].contains(req.method()) {
            panic!("BasePutOrPatchService must be routed to for only PUT / PATCH requests");
        }

        let (mut req_parts, body) = req.into_parts();

        // Get normal resource uri.
        let res_uri = req_parts
            .extensions
            .get::<NormalAbsoluteHttpUri>()
            .expect("BasePutOrPatchService must be called after confirming resource uri is normal absolute http uri.").clone();

        // Get content-type for attached payload.
        let payload_content_type = resolve_req_content_type(&req_parts.headers)?;
        // Resolve rep content-length-hint from content-length header.
        let payload_content_length_hint = resolve_req_content_length_hint(&req_parts.headers);

        // Resolve rep update action.
        let rep_update_action = if req_parts.method == Method::PUT {
            // If method is `PUT`, treat payload as representation data itself.
            RepUpdateAction::SetWith(
                BasicRepresentation {
                    metadata: RepresentationMetadata::new()
                        .with::<KContentType>(payload_content_type),
                    data: BytesStream::from_http_body(body, Some(payload_content_length_hint)),
                    base_uri: Some(res_uri.clone().into_subject()),
                }
                .into_binary(),
            )
        } else {
            // Else treat payload as patch.
            // Get resolved rep patcher.
            let resolved_rep_patcher = if let Some(pre_resolved) = req_parts
                .extensions
                .remove_rec_item::<KResolvedRepPatcher<Storage>>(
            ) {
                // If pre resolved by higher layers, then
                // return it.
                pre_resolved
            } else {
                // Resolve from components.
                Self::resolve_rep_patcher(
                    storage.clone(),
                    body,
                    payload_content_type,
                    res_uri.clone(),
                )
                .await?
            };
            RepUpdateAction::PatchWith(resolved_rep_patcher)
        };

        // Get res lock name.
        let res_lock_name = storage
            .repo()
            .uri_policy()
            .mutex_normal_res_uri_hash(&res_uri);

        // Compute upsert scope space.
        // By default, entire storage space.
        let upsert_scope_space = Arc::new(RelativeSolidStorageSpace {
            base_res_slot_id: SolidResourceSlotId::root_slot_id(storage.space().clone()),
        });

        // Construct resource upsert params
        let res_upsert_params = ResourceUpsertParams::try_new(
            res_uri.clone(),
            // Create or_slot_path by decoding from target
            //uri with hierarchical uri semantics.
            // TODO Must take from header in generalized model.
            SansAuxLinkRelativeResourceSlotPath::decode_with_hierarchical_uri_semantics(
                SolidResourceSlotId {
                    space: upsert_scope_space,
                    uri: res_uri,
                },
            )
            .map_err(|e| {
                error!("Routing violation.");
                INTERNAL_ERROR
                    .new_api_error_builder(StatusCode::INTERNAL_SERVER_ERROR)
                    .source(e)
                    .finish()
            })?,
            rep_update_action,
        )
        .expect("Must be valid upsert params.");

        // Resolve preconditions.
        let preconditions = HttpPreconditions {
            method: req_parts.method,
            preconditions: etag_base_normalized_conditional_headers(&req_parts.headers),
        };

        // Perform upsert with exclusive lock on request target.
        storage
            .resource_locker()
            .poll_with_lock(
                Self::conditional_upsert(
                    storage.clone(),
                    res_upsert_params,
                    preconditions,
                    req_parts
                        .extensions
                        .remove::<SgCredentials<Storage>>()
                        .unwrap_or_default(),
                    req_parts
                        .extensions
                        .remove_rec_item::<KOpReqExtensions>()
                        .unwrap_or_default(),
                    false,
                ),
                Some(res_lock_name),
                LockKind::Exclusive,
            )
            .await
    }

    /// Map the inner upsert problem to api-error
    fn map_upsert_problem(e: Problem) -> ApiError {
        // If access is denied.
        if ACCESS_DENIED.is_type_of(&e) {
            error!("Access denied.");
            ApiError::builder(StatusCode::FORBIDDEN).message("Not authorized.")
        } else if PRECONDITIONS_NOT_SATISFIED.is_type_of(&e) {
            ApiError::builder(
                resolve_preconditions_eval_status(&e).unwrap_or(StatusCode::PRECONDITION_FAILED),
            )
            .extend_with_opt::<KEvaluatedRepValidators>(
                e.extensions().get_rv::<KEvaluatedRepValidators>().cloned(),
            )
        }
        // On uri policy violation.
        else if URI_POLICY_VIOLATION.is_type_of(&e) {
            ApiError::builder(StatusCode::BAD_REQUEST).message("Uri policy violation.")
        }
        // If rdf source representation is invalid.
        else if INVALID_RDF_SOURCE_REPRESENTATION.is_type_of(&e) {
            ApiError::builder(StatusCode::BAD_REQUEST).message("Invalid rdf source representation.")
        }
        // If target is container, and supplied rep has
        // containment triples.
        else if INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES.is_type_of(&e) {
            SpecProblem::new(StatusCode::CONFLICT)
                .with_recourse_as_per(&REQ_SERVER_PROTECT_CONTAINMENT)
                .into()
        }
        // If target is container, and supplied rep has
        // contained resource metadata triples.
        else if INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA.is_type_of(&e) {
            SpecProblem::new(StatusCode::CONFLICT)
                .with_recourse_as_per(&REQ_SERVER_PROTECT_CONTAINED_RESOURCE_METADATA)
                .into()
        }
        // If supplied patch format is incompatible with
        // target rep content-type..
        else if INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE.is_type_of(&e) {
            ApiError::builder(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                .message("Incompatible patch target content-type.")
        }
        // If target resource rep is invalid encoded.
        else if INVALID_ENCODED_SOURCE_REP.is_type_of(&e) {
            ApiError::builder(StatusCode::CONFLICT)
                .message("Target resource representation is invalid encoded.")
        }
        // If supplied patch is invalid.
        else if PATCH_SEMANTICS_ERROR.is_type_of(&e) {
            // TODO generalized spec problem.
            let mut builder = ApiErrorBuilder::from(
                SpecProblem::new(StatusCode::UNPROCESSABLE_ENTITY)
                    .with_recourse_as_per(&REQ_SERVER_PATCH_N3_INVALID),
            );
            if let Some(source) = e.source() {
                builder = builder.extend_with::<KPatchErrorContext>(source.to_string());
            }
            builder
        }
        // If rep media type is not supported.
        else if UNSUPPORTED_MEDIA_TYPE.is_type_of(&e) {
            ApiError::builder(StatusCode::UNSUPPORTED_MEDIA_TYPE)
        }
        // If payload is too large.
        else if PAYLOAD_TOO_LARGE.is_type_of(&e) {
            ApiError::builder(StatusCode::PAYLOAD_TOO_LARGE)
        }
        // If operation is not supported.
        else if UNSUPPORTED_OPERATION.is_type_of(&e) {
            error!("Unsupported operation.");
            ApiError::builder(StatusCode::METHOD_NOT_ALLOWED)
        }
        // On any other error.
        else {
            ApiError::builder(StatusCode::INTERNAL_SERVER_ERROR)
        }
        .finish()
    }

    /// Upsert resource conditionally, and map any error to
    /// end user errors.
    #[tracing::instrument(skip_all, name = "BasePutOrPatchService::conditional_upsert", fields(
        res_uri = res_upsert_params.res_uri.as_str(),
        preconditions,
        skip_update,
    ))]
    async fn conditional_upsert(
        storage: Arc<Storage>,
        res_upsert_params: ResourceUpsertParams<Storage::Repo>,
        preconditions: HttpPreconditions,
        credentials: SgCredentials<Storage>,
        op_req_extensions: ClonableTypedRecord,
        skip_update: bool,
    ) -> Result<BasePutOrPatchResponse<Storage>, ApiError> {
        let (res_uri, or_slot_path, rep_update_action) = res_upsert_params.into_parts();

        let uri_policy = storage.repo().uri_policy();

        // Resolve status token.
        let status_token: SgResourceStatusToken<Storage> =
            resolve_status_token(storage.as_ref(), res_uri.clone()).await?;

        // Resolve effective operation based on resource's
        // existence.
        match status_token {
            ResourceStatusToken::NonExisting(ne_token) => match ne_token {
                NonExistingResourceToken::MutexExisting(_ne_me_token) => {
                    error!("Resource doesn't exist. But it's mutex resource does.");
                    // TODO should be different generalized requirement.
                    // TODO should attach context.
                    Err(SpecProblem::new(StatusCode::CONFLICT)
                        .with_recourse_as_per(&REQ_SERVER_URI_TRAILING_SLASH_DISTINCT)
                        .with_spec_problem_detail(
                            "Mutex resource exists for target resource or an immediate container.",
                        )
                        .into())
                }
                NonExistingResourceToken::MutexNonExisting(ne_mne_token) => {
                    info!("Resource doesn't exist, and no conflicting mutex resource exists.");
                    info!("Performing conditional create operation.");

                    // Evaluate pre conditions for non
                    // existing resource We are evaluating
                    // before-hand, as we have to decide flow
                    // for compound operation.
                    let pc_resolved_action = preconditions.evaluate_raw(None).0;

                    // If preconditions not satisfied, then return
                    if let PreconditionsResolvedAction::Return(status) = pc_resolved_action {
                        error!("Preconditions not satisfied.");
                        // Should have been 412
                        return Err(ApiError::builder(status)
                            .extend_with::<KEvaluatedRepValidators>(None)
                            .finish());
                    };

                    info!("Preconditions satisfied");

                    // Create intermediate containers if
                    // required. And resolve token for
                    // container of the new resource.

                    // Ensure repo uri-policy is honoured.
                    if !uri_policy.is_allowed_relative_slot_path(&or_slot_path) {
                        error!("Repo uri policy violation.");
                        return Err(Self::map_upsert_problem(URI_POLICY_VIOLATION.new_problem()));
                    }

                    let new_res_kind = or_slot_path.target_res_slot().res_kind();

                    let container_slot_path = or_slot_path.rsplit().0.ok_or_else(|| {
                        error!("Method scope root does not exist.");
                        ApiError::builder(StatusCode::BAD_REQUEST)
                            .message("Method scope root does not exist.")
                            .finish()
                    })?;

                    let container_lock_name =
                        uri_policy.mutex_normal_res_uri_hash(container_slot_path.target_res_uri());

                    // Proceed with an exclusive lock on
                    // immediate container.
                    let create_resp = storage
                        .resource_locker()
                        .poll_with_lock(
                            Self::create_nested(
                                storage.clone(),
                                ne_mne_token,
                                new_res_kind,
                                rep_update_action,
                                container_slot_path,
                                credentials,
                                op_req_extensions,
                            ),
                            Some(container_lock_name),
                            LockKind::Exclusive,
                        )
                        .await?;

                    Ok(BasePutOrPatchResponse {
                        upserted_res_slot: create_resp.created_resource_slot,
                        new_rep_validators: Self::optimistic_new_rep_validators(storage, res_uri)
                            .await?,
                        is_created: true,
                    })
                }
            },
            ResourceStatusToken::Existing(e_token) => {
                info!("Resource exists.");
                info!("Performing conditional update operation.");

                let res_slot = e_token.slot().clone();

                // If `skip_update` flag is set, return.
                if skip_update {
                    return Ok(BasePutOrPatchResponse {
                        upserted_res_slot: res_slot,
                        is_created: false,
                        new_rep_validators: None,
                    });
                }

                let mut update_fut = Box::pin(Self::conditional_update_existing(
                    e_token,
                    rep_update_action,
                    preconditions,
                    credentials,
                    op_req_extensions,
                ))
                    as BoxFuture<'static, Result<ResourceUpdateResponse, ApiError>>;

                // Lock the subject resource, if current
                // resource is an aux resource.
                if let Some(slot_rev_link) = res_slot.slot_rev_link() {
                    if slot_rev_link.rev_rel_type.is_auxiliary() {
                        let aux_subject_lock_name =
                            uri_policy.mutex_normal_res_uri_hash(&slot_rev_link.target);

                        update_fut = storage.resource_locker().poll_with_lock(
                            update_fut,
                            Some(aux_subject_lock_name),
                            LockKind::Exclusive,
                        );
                    }
                }

                let _ = update_fut.await?;

                Ok(BasePutOrPatchResponse {
                    upserted_res_slot: res_slot,
                    new_rep_validators: Self::optimistic_new_rep_validators(storage, res_uri)
                        .await?,
                    is_created: false,
                })
            }
        }
    }

    /// Conditional update existing resource..
    #[tracing::instrument(skip_all, name = "BasePutOrPatchService::conditional_update_existing", fields(
        res_uri = res_e_token.slot().id().uri.as_str(),
        preconditions,
    ))]
    async fn conditional_update_existing(
        res_e_token: RepoExistingResourceToken<Storage::Repo>,
        rep_update_action: RepUpdateAction<Storage::Repo>,
        preconditions: HttpPreconditions,
        credentials: SgCredentials<Storage>,
        op_req_extensions: ClonableTypedRecord,
    ) -> Result<ResourceUpdateResponse, ApiError> {
        // Construct res update future
        let mut updater = SgResourceUpdater::<Storage>::default();
        let res_update_fut = async move {
            updater
                .ready()
                .and_then(|svc| {
                    svc.call(ResourceUpdateRequest::<Storage::Repo> {
                        tokens: ResourceUpdateTokenSet::new(res_e_token),
                        rep_update_action,
                        preconditions: Box::new(preconditions),
                        credentials,
                        extensions: op_req_extensions,
                    })
                })
                .await
        };

        res_update_fut.await.map_err(|problem| {
            let opt_resolved_access_control = problem
                .extensions()
                .get_rv::<KResolvedAccessControl<SgCredentials<Storage>>>()
                .cloned();
            let mut e = Self::map_upsert_problem(problem);
            if let Some(acl) = opt_resolved_access_control {
                e.extensions_mut()
                    .insert_rec_item::<KResolvedAccessControl<SgCredentials<Storage>>>(acl);
            }
            e
        })
    }

    /// Create resource at given containment path.
    /// If intermediate containers in given containment path
    /// doesn't exist, it must create them.
    #[tracing::instrument(skip_all, name = "BasePutOrPatchService::create_nested", fields(
        new_res_uri = new_res_cf_token.uri().as_str(),
        container_slot_path,
    ))]
    fn create_nested(
        storage: Arc<Storage>,
        new_res_cf_token: SgResourceConflictFreeToken<Storage>,
        new_res_kind: SolidResourceKind,
        new_res_rep_update_action: RepUpdateAction<Storage::Repo>,
        // Method-scope relative.
        container_slot_path: SansAuxLinkRelativeResourceSlotPath<'static, Storage::StSpace>,
        credentials: SgCredentials<Storage>,
        op_req_extensions: ClonableTypedRecord,
    ) -> BoxFuture<'static, Result<ResourceCreateResponse<Storage::Repo>, ApiError>> {
        Box::pin(async move {
            let container_uri = container_slot_path.target_res_uri().clone();

            // Ensure container exists and represented.
            Self::conditional_upsert(
                storage.clone(),
                ResourceUpsertParams::try_new(
                    container_uri.clone(),
                    container_slot_path,
                    RepUpdateAction::SetWith(
                        BasicRepresentation {
                            metadata: RepresentationMetadata::new()
                                .with::<KContentType>(TEXT_TURTLE.clone()),
                            data: BytesStream::from_http_body(
                                Body::empty(),
                                Some(SizeHint::with_exact(0)),
                            ),
                            base_uri: Some(container_uri.clone().into_subject()),
                        }
                        .into_binary(),
                    ),
                )
                .expect("Must be valid"),
                HttpPreconditions {
                    method: Method::PUT,
                    // No conditions.
                    preconditions: Default::default(),
                },
                credentials.clone(),
                op_req_extensions.clone(),
                // Must skip updating if exists.
                true,
            )
            .await?;

            // Re query container's status token.
            // TODO Operation can return token if possible.
            let container_er_token = (resolve_status_token(storage.as_ref(), container_uri.clone())
                .await? as SgResourceStatusToken<Storage>)
                .existing_represented()
                .ok_or_else(|| {
                    error!("Invariant error. Container was created just, and lock is still there.");
                    ApiError::builder(StatusCode::INTERNAL_SERVER_ERROR).finish()
                })?;

            // Ensure that host resource is really a container.
            // Ensure request target is indeed a container as
            // assumed.
            if !container_er_token.slot().is_container_slot() {
                error!("Intermediate host resource is not a container.");
                return Err(ApiError::builder(StatusCode::BAD_REQUEST)
                    .message("Intermediate host resource is not a container.")
                    .finish());
            }

            // Construct resource create request.
            let new_res_create_request = ResourceCreateRequest::<Storage::Repo> {
                tokens: ResourceCreateTokenSet::try_new(new_res_cf_token, container_er_token)
                    .expect("Must be repo consistent tokens."),
                resource_kind: new_res_kind,
                slot_rev_rel_type: SlotRelationType::Contains,
                rep_update_action: new_res_rep_update_action,
                host_preconditions: Box::new(()),
                credentials,
                extensions: op_req_extensions,
            };

            // Create resource.
            SgResourceCreator::<Storage>::default()
                .ready()
                .and_then(|svc| svc.call(new_res_create_request))
                .await
                .map_err(|problem| {
                    let opt_resolved_host_access_control = problem
                        .extensions()
                        .get_rv::<KResolvedHostAccessControl<SgCredentials<Storage>>>()
                        .cloned();
                    let mut e = Self::map_upsert_problem(problem);
                    if let Some(acl) = opt_resolved_host_access_control {
                        e.extensions_mut()
                            // TODO "ancestor" instead of host?
                            .insert_rec_item::<KResolvedHostAccessControl<SgCredentials<Storage>>>(
                                acl,
                            );
                    }
                    e
                })
        })
    }

    /// Resolve validators of new rep for a represented
    /// resource if possible. If there is any process errors,
    /// return None. MUST be called only for represented resources.
    #[tracing::instrument(skip_all, fields(res_uri))]
    async fn optimistic_new_rep_validators(
        storage: Arc<Storage>,
        res_uri: SolidResourceUri,
    ) -> Result<Option<RepresentationMetadata>, ApiError> {
        // Resolve status token.
        let er_token = if let Ok(status_token) =
            resolve_status_token(storage.as_ref(), res_uri.clone()).await
        {
            status_token.existing_represented().ok_or_else(|| {
                error!("PUT/PATCH invariant error. just updated rep doesn't exists.");
                ApiError::builder(StatusCode::INTERNAL_SERVER_ERROR).finish()
            })
        } else {
            warn!("Unknown error occurred in resolving new rep validators.");
            return Ok(None);
        }?;

        Ok(Some(er_token.rep_validators()))
    }

    /// Resolve rep patcher from patch body.
    #[tracing::instrument(skip_all, fields(res_uri, patch_content_type))]
    async fn resolve_rep_patcher(
        storage: Arc<Storage>,
        patch_body: Body,
        patch_content_type: MediaType,
        res_uri: SolidResourceUri,
    ) -> Result<SgRepPatcher<Storage>, ApiError> {
        // Construct patch rep.
        let patch_doc_rep = BasicRepresentation {
            metadata: RepresentationMetadata::default().with::<KContentType>(patch_content_type),
            data: BytesStream::from(patch_body),
            base_uri: Some(res_uri.into_subject()),
        }
        .into_binary();

        // Resolve rep patcher.
        let rep_patcher = storage
            .repo()
            .rep_patcher_resolver()
            .ready()
            .and_then(|s| s.call(patch_doc_rep))
            .await
            .map_err(|e| {
                error!("Error in resolving rep patcher.");
                // If patch payload is too large.
                if PAYLOAD_TOO_LARGE.is_type_of(&e) {
                    ApiError::builder(StatusCode::PAYLOAD_TOO_LARGE)
                        .message("Patch body is too large.")
                }
                // If patch document type is not known.
                else if UNKNOWN_PATCH_DOC_CONTENT_TYPE.is_type_of(&e) {
                    ApiError::builder(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                        .message("Unsupported patch document type.")
                }
                // If patch document is invalid encoded.
                else if INVALID_ENCODED_PATCH.is_type_of(&e) {
                    let mut builder = ApiError::builder(StatusCode::BAD_REQUEST)
                        .message("Invalid encoded patch document.");
                    if let Some(source) = e.source() {
                        builder = builder.field("patch_doc_error_context", source.to_string());
                    }
                    builder
                } else {
                    ApiError::builder(StatusCode::INTERNAL_SERVER_ERROR)
                }
                .finish()
            })?;

        info!("Rep patcher resolution success");

        Ok(rep_patcher)
    }
}

/// A typed record key for resolved rep patcher.
pub struct KResolvedRepPatcher<Storage> {
    _phantom: fn() -> Storage,
}

impl<Storage> Clone for KResolvedRepPatcher<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _phantom: self._phantom,
        }
    }
}

impl<Storage: SolidStorage> TypedRecordKey for KResolvedRepPatcher<Storage> {
    type Value = SgRepPatcher<Storage>;
}

/// A struct for representing resource upsert params.
#[derive(Debug)]
pub struct ResourceUpsertParams<R: Repo> {
    /// Uri of the resource to be upserted.
    res_uri: SolidResourceUri,

    /// slot path for creating resource if doesn't exist.
    /// Currently upsert only supports contained slots for
    /// resource creation.
    or_slot_path: SansAuxLinkRelativeResourceSlotPath<'static, R::StSpace>,

    /// Rep update action .
    rep_update_action: RepUpdateAction<R>,
}

/// Error for invalid upsert resource params.
#[derive(Debug, thiserror::Error)]
pub enum InvalidUpsertResourceParams {
    /// Resource uri conflict.
    #[error("Resource uri and or_slot_path conflict.")]
    ResourceUriSlotConflict,
}

impl<R: Repo> ResourceUpsertParams<R> {
    /// Try to create new [`ResourceUpsertParams`].
    pub fn try_new(
        res_uri: SolidResourceUri,
        or_slot_path: SansAuxLinkRelativeResourceSlotPath<'static, R::StSpace>,
        rep_action: RepUpdateAction<R>,
    ) -> Result<Self, InvalidUpsertResourceParams> {
        // Ensure res uri and slot path's target res uri
        // matches.
        if &res_uri != or_slot_path.target_res_uri() {
            return Err(InvalidUpsertResourceParams::ResourceUriSlotConflict);
        }
        Ok(Self {
            res_uri,
            or_slot_path,
            rep_update_action: rep_action,
        })
    }

    /// Convert into parts.
    #[inline]
    pub fn into_parts(
        self,
    ) -> (
        SolidResourceUri,
        SansAuxLinkRelativeResourceSlotPath<'static, R::StSpace>,
        RepUpdateAction<R>,
    ) {
        (self.res_uri, self.or_slot_path, self.rep_update_action)
    }
}

/// A key for nested mutex resource uri.
#[derive(Debug, Clone)]
pub struct KNestedMutexResourceUri;

impl TypedRecordKey for KNestedMutexResourceUri {
    type Value = NormalAbsoluteHttpUri;
}
