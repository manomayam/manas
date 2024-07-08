//! I define routing service that routes to a matched service based on request method.
//!

use std::{
    collections::HashMap,
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};

use dyn_clone::clone_box;
use http::{Method, Request, Response, StatusCode};
use tower::{Service, ServiceExt};

use crate::{
    body::Body,
    service::{BoxHttpResponseFuture, HttpService},
};

/// A [`Service`] that routes http requests to registered
/// service corresponding to request method.
/// If no service is registered to handle a request method,
/// it will return error response with 405 METHOD_NOT_ALLOWED status.
pub struct RouteByMethod {
    method_services: Arc<HashMap<Method, Box<dyn HttpService<Body, Body>>>>,
    poll_ready: bool,
}

impl std::fmt::Debug for RouteByMethod {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteByMethod")
            .field("poll_ready", &self.poll_ready)
            .finish()
    }
}

impl Clone for RouteByMethod {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            method_services: self.method_services.clone(),
            poll_ready: self.poll_ready,
        }
    }
}

impl RouteByMethod {
    /// Get a new [`RouteByMethod`].
    #[inline]
    pub fn new(method_services: Arc<HashMap<Method, Box<dyn HttpService<Body, Body>>>>) -> Self {
        Self {
            method_services,
            poll_ready: false,
        }
    }
}

impl Service<Request<Body>> for RouteByMethod {
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.poll_ready {
            return Poll::Ready(Ok(()));
        }
        // If any of services is pending, return pending.
        // If any of services resolved with error, panic.
        for s in self.method_services.values() {
            if let Poll::Ready(result) = clone_box(s.as_ref()).poll_ready(cx) {
                assert!(result.is_ok(), "Method services must be infallible");
            } else {
                return Poll::Pending;
            }
        }

        // Or else, return ready.
        self.poll_ready = true;
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        if let Some(method_service) = self.method_services.get(req.method()) {
            // Get owned method service.
            let mut method_service = clone_box(method_service.as_ref());
            Box::pin(async move {
                // Get response from method service.
                let response: Response<Body> = method_service
                    .ready()
                    .await
                    .expect("Must be infallible")
                    .call(req)
                    .await
                    .expect("Method service must be infallible.");

                Ok(response)
            })
        } else {
            // Return error response with NOT_IMPLEMENTED status.
            // From [rfc9110](https://www.rfc-editor.org/rfc/rfc9110.html#section-9.1):
            // >  An origin server that receives a request method that is unrecognized
            // > or not implemented SHOULD respond with the 501 (Not Implemented) status code.
            Box::pin(futures::future::ready(Ok(Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Body::empty())
                .expect("Must be valid."))))
        }
    }
}
