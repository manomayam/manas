//! I define default implementation of [`BaseResponseMarshaller`](super::super::super::BaseResponseMarshaller)
//! for marshalling [`BaseDeleteService`](super::super::base::BaseDeleteService) responses.
//!

use std::{convert::Infallible, sync::Arc, task::Poll};

use http::{Response, StatusCode};
use http_api_problem::ApiError;
use manas_http::service::BoxHttpResponseFuture;
use manas_http::{body::Body, problem::ApiErrorExt};
use manas_repo::service::resource_operator::deleter::ResourceDeleteResponse;
use tower::Service;

use crate::{
    service::method::common::snippet::authorization::attach_authorization_context, SolidStorage,
};

/// Configuration for [DefaultBaseDeleteResponseMarshaller`].
#[derive(Debug, Clone, Default)]
pub struct DefaultBaseDeleteResponseMarshallerConfig {
    /// If dev mode is enabled.
    pub dev_mode: bool,
}

/// Default implementation of [`BaseResponseMarshaller`](super::super::super::BaseResponseMarshaller)
/// for marshalling [`BaseDeleteService`](super::super::base::BaseDeleteService) responses.
#[derive(Debug)]
pub struct DefaultBaseDeleteResponseMarshaller<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,

    /// Marshal config.
    pub marshal_config: DefaultBaseDeleteResponseMarshallerConfig,
}

impl<Storage: SolidStorage> Clone for DefaultBaseDeleteResponseMarshaller<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            marshal_config: self.marshal_config.clone(),
        }
    }
}

impl<Storage> DefaultBaseDeleteResponseMarshaller<Storage> {
    /// Get new [`DefaultBaseDeleteResponseMarshaller`].
    #[inline]
    pub fn new(
        storage: Arc<Storage>,
        marshal_config: DefaultBaseDeleteResponseMarshallerConfig,
    ) -> Self {
        Self {
            storage,
            marshal_config,
        }
    }
}

impl<Storage> Service<Result<ResourceDeleteResponse<Storage::Repo>, ApiError>>
    for DefaultBaseDeleteResponseMarshaller<Storage>
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
    #[tracing::instrument(skip_all, name = "DefaultBaseDeleteResponseMarshaller::call")]
    fn call(
        &mut self,
        req: Result<ResourceDeleteResponse<Storage::Repo>, ApiError>,
    ) -> Self::Future {
        Box::pin(futures::future::ready(Ok(match req {
            Ok(resp) => self.marshal_ok(resp),
            Err(err) => self.marshal_err(err),
        })))
    }
}

impl<Storage> DefaultBaseDeleteResponseMarshaller<Storage>
where
    Storage: SolidStorage,
{
    /// Marshal `BaseDelete`Service success response.
    pub fn marshal_ok(&self, _response: ResourceDeleteResponse<Storage::Repo>) -> Response<Body> {
        // Create builder, set status code to 204.
        let builder = Response::builder().status(StatusCode::NO_CONTENT);

        // TODO serialize the response.

        // Set empty body, and return response.
        builder
            .body(Body::empty())
            .expect("Must be valid hyper response.")
    }

    /// Marshal `BaseDeleteService` error.
    pub fn marshal_err(&self, mut error: ApiError) -> Response<Body> {
        if self.marshal_config.dev_mode {
            attach_authorization_context::<Storage>(&mut error);
        }
        error.into_http_response()
    }
}
