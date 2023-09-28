//! I define traits for http services in manas ecosystem.
//!

use std::convert::Infallible;

use dyn_clone::{clone_trait_object, DynClone};
use futures::future::BoxFuture;
use http::{Request, Response};
use tower::Service;

pub mod impl_;
pub mod namespaced;

/// A future for infallible http response..
pub type BoxHttpResponseFuture<ResBody> = BoxFuture<'static, Result<Response<ResBody>, Infallible>>;

/// A trait for infallible http services.
pub trait HttpService<ReqBody, ResBody>:
    Service<
        Request<ReqBody>,
        Response = Response<ResBody>,
        Error = Infallible,
        Future = BoxHttpResponseFuture<ResBody>,
    > + DynClone
    + Send
    + Sync
    + 'static
{
}

impl<ReqBody, ResBody, S> HttpService<ReqBody, ResBody> for S where
    S: Service<
            Request<ReqBody>,
            Response = Response<ResBody>,
            Error = Infallible,
            Future = BoxHttpResponseFuture<ResBody>,
        > + DynClone
        + Send
        + Sync
        + 'static
{
}

clone_trait_object!(<ReqBody, ResBody> HttpService<ReqBody, ResBody>);
