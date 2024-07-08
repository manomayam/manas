//! I define service implementations for
//! http methods on solid resources.
//!

use std::{
    convert::Infallible,
    task::{Context, Poll},
};

use futures::{future::BoxFuture, FutureExt};
use http::{Request, Response};
use http_api_problem::ApiError;
use manas_http::body::Body;
use manas_http::service::BoxHttpResponseFuture;
use tower::{Service, ServiceExt};

pub mod common;

pub mod delete;
pub mod get;
pub mod head;
pub mod post;
pub mod put_or_patch;

/// [`MethodService`] handles requests on resources in a storage space in two phases.
///
/// 1. Configured base method svc takes hyper request, processes it,
/// and returns custom success response or an `ApiError`.
///
/// 2. Then marshaller marshals either success response of base svc or api error into hyper response infallibly.
///
/// This phased handling of a request allows for middleware
/// at different stages of request.
///
/// 1. If one wants to customize  base request handling and base response, then they
/// can layer over base method service, and plug resultant.
///
/// 2. If one want to customize how base response is marshalled,
/// then they can create custom marshaller by layering over existing ones.
///
/// 3. If one want to customize final hyper response or initial hyper request,
/// then they can layer over this service to create custom
/// method services.
#[derive(Debug, Clone)]
pub struct MethodService<BaseMethodSvc, Marshaller> {
    /// Base method service
    pub base_method_svc: BaseMethodSvc,

    /// Base method response marshaller.
    pub marshaller: Marshaller,
}

impl<BaseMethodSvc, Marshaller> Service<Request<Body>> for MethodService<BaseMethodSvc, Marshaller>
where
    BaseMethodSvc: BaseMethodService,
    Marshaller: BaseResponseMarshaller<BaseMethodSvc::BaseResponse>,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let marshaller = self.marshaller.clone();
        Box::pin(
            self.base_method_svc
                .clone()
                .oneshot(req)
                .then(move |base_result| marshaller.oneshot(base_result)),
        )
    }
}

/// A base method svc takes hyper request, and returns custom success response or an `ApiError`.
pub trait BaseMethodService:
    Service<
        Request<Body>,
        Response = Self::BaseResponse,
        Error = ApiError,
        Future = BoxFuture<'static, Result<Self::BaseResponse, ApiError>>,
    > + Clone
    + Send
    + 'static
{
    /// Type of response of this base method service.
    type BaseResponse: Send + 'static;
}

impl<BR, S> BaseMethodService for S
where
    S: Service<
            Request<Body>,
            Response = BR,
            Error = ApiError,
            Future = BoxFuture<'static, Result<BR, ApiError>>,
        > + Clone
        + Send
        + 'static,
    BR: Send + 'static,
{
    type BaseResponse = BR;
}

/// A marshaller marshals either success response of base svc or api error into hyper response infallibly.
pub trait BaseResponseMarshaller<R>:
    Service<
        Result<R, ApiError>,
        Response = Response<Body>,
        Error = Infallible,
        Future = BoxHttpResponseFuture<Body>,
    > + Clone
    + Send
    + 'static
{
}

impl<BR, S> BaseResponseMarshaller<BR> for S where
    S: Service<
            Result<BR, ApiError>,
            Response = Response<Body>,
            Error = Infallible,
            Future = BoxHttpResponseFuture<Body>,
        > + Clone
        + Send
        + 'static
{
}
