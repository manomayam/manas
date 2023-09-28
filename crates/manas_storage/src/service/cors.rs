//! I define middleware to handle cors semantics.
//!

use std::convert::Infallible;

use http::{
    header::{
        ACCEPT, ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ALLOW,
        AUTHORIZATION, ETAG, LAST_MODIFIED, LINK, LOCATION, ORIGIN, WWW_AUTHENTICATE,
    },
    HeaderName, Request, Response,
};
use hyper::Body;
use manas_http::{
    header::{
        accept_patch::ACCEPT_PATCH, accept_post::ACCEPT_POST, accept_put::ACCEPT_PUT,
        preference_applied::PREFERENCE_APPLIED, wac_allow::WAC_ALLOW,
    },
    service::BoxHttpResponseFuture,
};
use tower::{Layer, Service};
use tower_http::cors::{Cors, CorsLayer};

/// A middleware to handle cors semantics in liberal way.
///
/// From spec:
/// A server MUST implement the CORS protocol such that,
/// to the extent possible, the browser allows Solid apps to
/// send any request and combination of request headers to the
/// server, and the Solid app can read any response and response
/// headers received from the server.
///
#[derive(Debug, Clone)]
pub struct LiberalCors<S> {
    inner: Cors<S>,
}

impl<S> LiberalCors<S> {
    /// Wrap a given service to get middleware applied service.
    pub fn new(inner: S) -> Self {
        let cors_layer = CorsLayer::very_permissive()
            .expose_headers([
                ACCEPT_PATCH.clone(),
                ACCEPT_POST.clone(),
                ACCEPT_PUT.clone(),
                ALLOW,
                ETAG,
                LAST_MODIFIED,
                LINK,
                LOCATION,
                PREFERENCE_APPLIED.clone(),
                WWW_AUTHENTICATE,
                WAC_ALLOW.clone(),
                HeaderName::from_static("updates-via"),
            ])
            // Cors layer currently overrides any inner vary.
            // So we cannot preserve inner value for now.
            .vary([
                ORIGIN,
                ACCEPT,
                AUTHORIZATION,
                ACCESS_CONTROL_REQUEST_METHOD,
                ACCESS_CONTROL_REQUEST_HEADERS,
            ]);

        // TODO allow custom config.

        Self {
            inner: cors_layer.layer(inner),
        }
    }
}

impl<S> Service<Request<Body>> for LiberalCors<S>
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible>,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        Box::pin(self.inner.call(req))
    }
}
