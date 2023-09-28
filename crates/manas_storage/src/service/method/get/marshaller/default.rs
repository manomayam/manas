//! I define default implementation of [`BaseResponseMarshaller`](super::super::super::BaseResponseMarshaller)
//! for marshalling [`BaseGetService`](super::super::base::BaseGetService) responses.
//!

use std::{convert::Infallible, sync::Arc, task::Poll};

use headers::{AcceptRanges, ContentLength, ContentType, ETag, HeaderMapExt};
use http::{header::VARY, Response, StatusCode};
use http_api_problem::ApiError;
use hyper::Body;
use if_chain::if_chain;
use iri_string::types::UriReferenceStr;
use manas_access_control::model::KResolvedAccessControl;
use manas_http::{
    header::link::{Link, LinkRel, LinkTarget, LinkValue},
    representation::{
        metadata::{KCompleteContentLength, KContentRange, KDerivedETag, KLastModified},
        Representation, RepresentationExt,
    },
    service::BoxHttpResponseFuture,
};
use manas_repo::service::resource_operator::common::preconditions::KEvaluatedRepValidators;
use manas_space::{resource::slot::SolidResourceSlot, SolidStorageSpace};
use rdf_vocabularies::ns;
use tower::Service;
use typed_record::TypedRecord;

use crate::{
    policy::method::MethodPolicyExt,
    service::method::{
        common::snippet::authorization::attach_authorization_context,
        get::base::{error_context::KExistingMutexResourceUri, BaseGetResponse},
    },
    SgCredentials, SolidStorage,
};

/// Configuration fr [`DefaultBaseGetResponseMarshaller`].
#[derive(Debug, Clone, Default)]
pub struct DefaultBaseGetResponseMarshallerConfig {
    /// Whether to redirect if mutex resource exists, instead of error.
    pub redirect_if_mutex_resource_exists: bool,

    /// If dev mode is enabled.
    pub dev_mode: bool,
}

/// Default implementation of [`BaseResponseMarshaller`](super::super::super::BaseResponseMarshaller)
/// for marshalling [`BaseGetService`](super::super::base::BaseGetService) responses.
#[derive(Debug)]
pub struct DefaultBaseGetResponseMarshaller<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,

    /// Marshal configuration.
    pub marshal_config: DefaultBaseGetResponseMarshallerConfig,
}

impl<Storage> Clone for DefaultBaseGetResponseMarshaller<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            marshal_config: self.marshal_config.clone(),
        }
    }
}

impl<Storage> DefaultBaseGetResponseMarshaller<Storage> {
    /// Get new [`DefaultBaseGetResponseMarshaller`].
    #[inline]
    pub fn new(
        storage: Arc<Storage>,
        marshal_config: DefaultBaseGetResponseMarshallerConfig,
    ) -> Self {
        Self {
            storage,
            marshal_config,
        }
    }
}

impl<Storage> Service<Result<BaseGetResponse<Storage>, ApiError>>
    for DefaultBaseGetResponseMarshaller<Storage>
where
    Storage: SolidStorage,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "DefaultBaseGetResponseMarshaller::call")]
    fn call(&mut self, req: Result<BaseGetResponse<Storage>, ApiError>) -> Self::Future {
        Box::pin(futures::future::ready(Ok(match req {
            Ok(resp) => self.marshal_ok(resp),
            Err(err) => self.marshal_err(err),
        })))
    }
}

impl<Storage> DefaultBaseGetResponseMarshaller<Storage>
where
    Storage: SolidStorage,
{
    /// Marshall success base get response.
    #[allow(clippy::vec_init_then_push)]
    pub fn marshal_ok(&self, base_response: BaseGetResponse<Storage>) -> Response<Body> {
        let rep = base_response.state.representation();
        let res_slot: &SolidResourceSlot<Storage::StSpace> = &base_response.state.slot;

        // Get response builder with resolved status code.
        let mut builder = Response::builder();

        // Set resolved status code.
        builder = builder.status(if rep.is_complete() {
            StatusCode::OK
        } else {
            StatusCode::PARTIAL_CONTENT
        });

        let headers = builder.headers_mut().expect("Must be valid");

        // Set rep  derived headers.

        // Allow, Accept-<Method>
        self.storage
            .method_policy()
            .set_allow_accept_headers_for_existing(
                headers,
                res_slot,
                rep.metadata().content_type(),
            );

        // Content-Type.
        headers.typed_insert::<ContentType>(rep.metadata().content_type().clone().into());

        // Content-Length and Content-Range
        if let Some(content_range) = rep.metadata().get_rv::<KContentRange>() {
            // If rep is partial,
            // Content-Range
            headers.typed_insert(content_range.clone());
            // Content-Length
            if let Some(bytes_range) = content_range.bytes_range() {
                headers.typed_insert(ContentLength(bytes_range.1 - bytes_range.0 + 1));
            }
        } else {
            // If rep is complete,
            // Content-Length
            if let Some(content_length) = rep.metadata().get_rv::<KCompleteContentLength>().cloned()
            {
                headers.typed_insert(content_length);
            }
        }

        // Accept-Ranges.
        // TODO should be configurable.
        headers.typed_insert(AcceptRanges::bytes());

        // Last-Modified
        if let Some(last_modified) = rep.metadata().get_rv::<KLastModified>().copied() {
            headers.typed_insert(last_modified);
        }

        //  ETag
        if let Some(etag) = rep.metadata().get_rv::<KDerivedETag>().cloned() {
            headers.typed_insert::<ETag>(etag.into());
        }
        // List of links.
        let mut links = Vec::<LinkValue>::new();

        // Set headers specified by ldp

        // Set ldp resource types.
        links.push(
            LinkValue::try_new_basic(ns::ldp::Resource.to_string(), "type").expect("Must be valid"),
        );
        // If is container.
        if res_slot.is_container_slot() {
            links.push(
                LinkValue::try_new_basic(ns::ldp::BasicContainer.to_string(), "type")
                    .expect("Must be valid"),
            );
        }

        // Set headers specified by solid protocol.

        let space = res_slot.space();

        // Set slot reverse links.
        if let Some(slot_rev_link) = res_slot.slot_rev_link() {
            links.push(LinkValue::new(
                // Target as resource itself
                LinkTarget(AsRef::<UriReferenceStr>::as_ref(&*res_slot.id().uri).to_owned()),
                LinkRel::new(slot_rev_link.rev_rel_type.clone().into()),
                // Host resource as anchor.
                Some(AsRef::<UriReferenceStr>::as_ref(&*slot_rev_link.target).to_owned()),
            ))
        }

        // Push storage description link.
        // Req: Servers MUST include the Link header with
        // rel="http://www.w3.org/ns/solid/terms#storageDescription"
        // targeting the URI of the storage description resource
        // in the response of HTTP GET, HEAD and OPTIONS
        // requests targeting a resource in a storage.
        links.push(
            LinkValue::try_new_basic(
                space.description_res_uri().as_str(),
                "http://www.w3.org/ns/solid/terms#storageDescription",
            )
            .expect("Must be valid"),
        );

        // If resource is storage root container,
        if res_slot.is_root_slot() {
            // Push storage type link.
            // Req: Servers exposing the storage resource MUST
            // advertise by including the HTTP Link header with rel="type"
            // targeting http://www.w3.org/ns/pim/space#Storage
            // when responding to storage’s request URI.
            links.push(
                LinkValue::try_new_basic(ns::pim::Storage.to_string(), "type")
                    .expect("Must be valid"),
            );

            // Push storage owner link.
            // Req: When a server wants to advertise the owner
            // of a storage, the server MUST include the Link
            // header with rel="http://www.w3.org/ns/solid/terms#owner"
            // targeting the URI of the owner in the response of
            // HTTP HEAD or GET requests targeting the root container.
            links.push(
                LinkValue::try_new_basic(space.owner_id().as_str(), ns::solid::owner.to_string())
                    .expect("Must be valid"),
            );
        }

        // Push links to auxiliary resources.
        // Req: Servers MUST advertise auxiliary resources
        // associated with a subject resource by responding to
        // HEAD and GET requests by including the HTTP Link
        // header with the rel parameter [RFC8288].
        for item in &base_response.aux_links_index {
            links.push(item.clone().into())
        }

        // Set link header with collected links.
        headers.typed_insert(Link { values: links });

        // TODO customizable?
        headers.insert(
            VARY,
            "Accept, Authorization, Origin"
                .parse()
                .expect("Must be valid"),
        );

        // Set Wac-Allow.
        if let Some(resolved_acl) = base_response
            .extensions
            .get_rv::<KResolvedAccessControl<SgCredentials<Storage>>>()
        {
            headers.typed_insert(resolved_acl.authorization().to_wac_allow());
        }

        builder
            .body(Body::wrap_stream(
                base_response
                    .state
                    .into_parts()
                    .1
                    .into_basic()
                    .into_streaming()
                    .data
                    .stream,
            ))
            .expect("Must be well formed response")
    }

    /// Marshall base get error.
    pub fn marshal_err(&self, mut error: ApiError) -> Response<Body> {
        if self.marshal_config.dev_mode {
            attach_authorization_context::<Storage>(&mut error);
        }

        // Get hyper response.
        let mut response = error.to_http_api_problem().to_hyper_response();

        if response.status() == StatusCode::NOT_FOUND {
            // Check if mutex resource exists, and redirect instead of error if configured.
            if_chain! {
                // If configured to redirect,
                if self.marshal_config.redirect_if_mutex_resource_exists;
                // If mutex resource exists.
                if let Some(mutex_res_uri) = error.extensions().get_rv::<KExistingMutexResourceUri>();

                then {
                    // Override response.
                    // Req: Instead, the server MAY respond to requests for the latter URI with a 301 redirect to the former.
                    response = Response::builder()
                        .status(StatusCode::MOVED_PERMANENTLY)
                        .header("Location", mutex_res_uri.as_str())
                        .body(Body::empty())
                        .expect("Must be valid");
                }

                // If 404, set Allow, Accept-<Method>
                else {
                    self.storage
                        .method_policy()
                        .set_allow_accept_headers_for_non_existing(response.headers_mut());
                }
            };
        }

        // If 304, set etag,vary, Wac-Allow.
        // Req: The server generating a 304 response MUST
        // generate any of the following header fields that
        // would have been sent in a 200 (OK) response to the
        // same request: Cache-Control, Content-Location,
        // Date, ETag, Expires, and Vary.
        if response.status() == StatusCode::NOT_MODIFIED {
            if_chain! {
                if let Some(Some(rep_validators)) = error.extensions().get_rv::<KEvaluatedRepValidators>();
                if let Some(etag) = rep_validators.get_rv::<KDerivedETag>().cloned();

                then {
                    response.headers_mut().typed_insert::<ETag>(etag.into())
                }
            };

            // TODO customizable?
            response.headers_mut().insert(
                VARY,
                "Accept, Authorization, Origin"
                    .parse()
                    .expect("Must be valid"),
            );

            // Set Wac-Allow.
            // @See: https://github.com/CommunitySolidServer/CommunitySolidServer/issues/1676
            if let Some(resolved_acl) = error
                .extensions()
                .get_rv::<KResolvedAccessControl<SgCredentials<Storage>>>()
            {
                response
                    .headers_mut()
                    .typed_insert(resolved_acl.authorization().to_wac_allow());
            }
        }

        response
    }
}
