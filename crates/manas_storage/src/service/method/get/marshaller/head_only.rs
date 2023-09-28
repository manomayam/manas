//! I define an implementation of [`BaseResponseMarshaller`]
//! for marshalling [`BaseGetService`](super::super::base::BaseGetService) responses  for head requests.
//!

use std::{convert::Infallible, marker::PhantomData, task::Poll};

use futures::TryFutureExt;
use http::Response;
use http_api_problem::ApiError;
use hyper::Body;
use manas_http::service::BoxHttpResponseFuture;
use tower::Service;

use crate::{
    service::method::{get::base::BaseGetResponse, BaseResponseMarshaller},
    SolidStorage,
};

/// [`HeadOnlyBaseGetResponseMarshaller`] delegates marshalling to configured inner marshaller,
/// and strips off any representation body from marshalled response.
#[derive(Debug)]
pub struct HeadOnlyBaseGetResponseMarshaller<Storage, Inner> {
    inner: Inner,
    _phantom: PhantomData<fn() -> Storage>,
}

impl<Storage, Inner: Clone> Clone for HeadOnlyBaseGetResponseMarshaller<Storage, Inner> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<Space, Inner> HeadOnlyBaseGetResponseMarshaller<Space, Inner> {
    /// Get a new [`HeadOnlyBaseGetResponseMarshaller`].
    #[inline]
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<Storage, Inner> Service<Result<BaseGetResponse<Storage>, ApiError>>
    for HeadOnlyBaseGetResponseMarshaller<Storage, Inner>
where
    Storage: SolidStorage,
    Inner: BaseResponseMarshaller<BaseGetResponse<Storage>>,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "HeadOnlyBaseGetResponseMarshaller::call")]
    fn call(&mut self, req: Result<BaseGetResponse<Storage>, ApiError>) -> Self::Future {
        Box::pin(self.inner.call(req).map_ok(|resp| {
            // Replace body with empty body.
            let (parts, _body) = resp.into_parts();
            Response::from_parts(parts, hyper::Body::empty())
        }))
    }
}
