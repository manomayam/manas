//! I provide few adapters to use in hyper ecosystem.
//!

use std::{
    marker::PhantomData,
    task::{Context, Poll},
};

use http::{Request, Response};
use hyper::body::Incoming;
use tower::Service;

use crate::body::Body;

/// A middleware [`Service`] that adapts the request body..
///
#[derive(Debug)]
pub struct AdaptIncomingBody<S, ResBody>
where
    S: Clone,
{
    inner: S,
    _phantom: PhantomData<fn() -> ResBody>,
}

impl<S: Clone, ResBody> Clone for AdaptIncomingBody<S, ResBody>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<S, ResBody> AdaptIncomingBody<S, ResBody>
where
    S: Clone,
{
    /// Create a new [`AdaptReqBody`].
    #[inline]
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<S, ResBody> Service<Request<Incoming>> for AdaptIncomingBody<S, ResBody>
where
    S: Service<Request<Body>, Response = Response<ResBody>> + Clone,
{
    type Response = Response<ResBody>;

    type Error = S::Error;

    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req_adapted = Request::from_parts(parts, Body::new(body));
        self.inner.call(req_adapted)
    }
}
