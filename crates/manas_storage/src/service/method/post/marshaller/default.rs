//! I define default implementation of [`BaseResponseMarshaller`]((super::super::super::BaseResponseMarshaller))
//! for marshalling [`BasePostService`](super::super::base::BasePostService) responses.
//!

use std::{convert::Infallible, sync::Arc, task::Poll};

use headers::HeaderMapExt;
use http::{Response, StatusCode};
use http_api_problem::ApiError;
use hyper::Body;
use manas_http::{
    header::link::{Link, LinkValue},
    service::BoxHttpResponseFuture,
};
use manas_space::resource::kind::SolidResourceKind;
use rdf_vocabularies::ns;
use tower::Service;

use crate::{
    service::method::{
        common::snippet::authorization::attach_authorization_context, post::base::BasePostResponse,
    },
    SolidStorage,
};

/// Configuration for [`DefaultBasePostResponseMarshaller`].
#[derive(Debug, Clone, Default)]
pub struct DefaultBasePostResponseMarshallerConfig {
    /// If dev mode is enabled.
    pub dev_mode: bool,
}

/// Default implementation of [`BaseResponseMarshaller`](super::super::super::BaseResponseMarshaller)
/// for marshalling [`BasePostService`](super::super::base::BasePostService) responses.
#[derive(Debug)]
pub struct DefaultBasePostResponseMarshaller<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,

    /// Marshal configuration.
    pub marshal_config: DefaultBasePostResponseMarshallerConfig,
}

impl<Storage> Clone for DefaultBasePostResponseMarshaller<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            marshal_config: self.marshal_config.clone(),
        }
    }
}

impl<Storage> DefaultBasePostResponseMarshaller<Storage> {
    /// Get a new [`DefaultBasePostResponseMarshaller`] with given params.
    #[inline]
    pub fn new(
        storage: Arc<Storage>,
        marshal_config: DefaultBasePostResponseMarshallerConfig,
    ) -> Self {
        Self {
            storage,
            marshal_config,
        }
    }
}

impl<Storage> Service<Result<BasePostResponse<Storage>, ApiError>>
    for DefaultBasePostResponseMarshaller<Storage>
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
    #[tracing::instrument(skip_all, name = "DefaultBasePostResponseMarshaller::call")]
    fn call(&mut self, req: Result<BasePostResponse<Storage>, ApiError>) -> Self::Future {
        Box::pin(futures::future::ready(Ok(match req {
            Ok(resp) => self.marshal_ok(resp),
            Err(err) => self.marshal_err(err),
        })))
    }
}

impl<Storage> DefaultBasePostResponseMarshaller<Storage>
where
    Storage: SolidStorage,
{
    /// Marshal `BasePost` success response.
    #[allow(clippy::vec_init_then_push)]
    pub fn marshal_ok(&self, response: BasePostResponse<Storage>) -> Response<Body> {
        // Create builder, set status code to 201.
        let mut builder = Response::builder().status(StatusCode::CREATED);

        let headers = builder.headers_mut().expect("Must be valid headers");

        // Set `Location` header to uri of created resource.
        // Req: LDP-5.2.3.1: If the resource was created successfully, LDP servers must respond with status code 201 (Created) and the Location header set to the new resource’s URL.
        // TODO Location as typed-header.
        headers.insert(
            "Location",
            response
                .created_resource_slot
                .id()
                .uri
                .as_str()
                .parse()
                .expect("Must be valid header value"),
        );

        // List of links
        let mut links = Vec::<LinkValue>::new();

        // Set ldp resource types for newly created resource.
        links.push(
            LinkValue::try_new_basic(ns::ldp::Resource.to_string(), "type").expect("Must be valid"),
        );
        // If is container.
        if response.created_resource_slot.res_kind() == SolidResourceKind::Container {
            links.push(
                LinkValue::try_new_basic(ns::ldp::BasicContainer.to_string(), "type")
                    .expect("Must be valid"),
            );
        }

        // Set link header with collected links.
        headers.typed_insert(Link { values: links });

        // Set empty body, and return response.
        builder
            .body(Body::empty())
            .expect("Must be valid hyper response.")
    }

    /// Marshal `BasePost` error.
    pub fn marshal_err(&self, mut error: ApiError) -> Response<Body> {
        if self.marshal_config.dev_mode {
            attach_authorization_context::<Storage>(&mut error);
        }

        error.into_hyper_response()
    }
}
