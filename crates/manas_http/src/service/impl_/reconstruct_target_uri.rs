//! I define middleware service for reconstructing uri of the request target.
//!

use std::{
    ops::Deref,
    task::{Context, Poll},
};

use ecow::EcoString;
use futures::future::{self, Either, Ready};
use headers::{Header, HeaderMapExt, Host};
use http::{uri::Scheme, HeaderName, Request, Response, StatusCode};
use http_api_problem::ApiError;
use hyper::Body;
use iri_string::types::{UriAbsoluteStr, UriAbsoluteString, UriStr};
use tower::Service;
use tracing::{debug, error, info};

use crate::{
    header::{
        forwarded::Forwarded, x_forwarded_host::XForwardedHost, x_forwarded_proto::XForwardedProto,
    },
    uri::invariant::AbsoluteHttpUri,
};

/// Params for uri reconstruction.
#[derive(Debug, Clone)]
pub struct UriReconstructionParams {
    /// Default scheme.
    pub default_scheme: Scheme,

    /// Trusted proxy headers.
    pub trusted_proxy_headers: Vec<HeaderName>,
}

/// A [`Service`] that reconstructs absolute uri of
/// the request target, before forwarding to inner service.
/// It sets the result as [`AbsoluteHttpUri`] typed request
/// extension, if not present already.
///
/// From [rfc9110](https://www.rfc-editor.org/rfc/rfc9110.html#section-7.1):
///
/// > Upon receipt of a client's request, a server reconstructs
/// the target URI from the received components in accordance with
/// their local configuration and incoming connection context.
#[derive(Debug, Clone)]
pub struct ReconstructTargetUri<S>
where
    S: Clone,
{
    /// Inner service
    inner: S,

    /// Uri reconstruction params.
    params: UriReconstructionParams,
}

impl<S> ReconstructTargetUri<S>
where
    S: Clone,
{
    /// Create a new [`ReconstructTargetUri`] with given default scheme, and an inner service.
    #[inline]
    pub fn new(params: UriReconstructionParams, inner: S) -> Self {
        Self { inner, params }
    }
}

impl<S> Service<Request<Body>> for ReconstructTargetUri<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone,
{
    type Response = Response<Body>;

    type Error = S::Error;

    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        // If already set, then delegate to inner service.
        if req.extensions().get::<AbsoluteHttpUri>().is_some() {
            return Either::Left(self.inner.call(req));
        }

        // Construct original uri.

        // Initialize with defaults.
        let (mut scheme, mut authority) = (
            EcoString::from(self.params.default_scheme.as_str()),
            req.uri()
                // http2 includes authority in request target.
                .authority()
                .cloned()
                .map(Into::into)
                // http1 includes authority in `Host` header.
                .or_else(|| req.headers().typed_get::<Host>()),
        );

        let trusted_proxy_headers = &self.params.trusted_proxy_headers;

        // Update from `Forwarded` header.
        if trusted_proxy_headers.contains(Forwarded::name()) {
            if let Some(h_forwarded) = req.headers().typed_get::<Forwarded>() {
                debug!("Forwarded header present. {:?}", h_forwarded);

                if let Some(forwarded_host) = h_forwarded.elements[0].host_decoded() {
                    authority = Some(forwarded_host);
                }

                if let Some(forwarded_proto) = h_forwarded.elements[0].proto() {
                    scheme = forwarded_proto.deref().into()
                }
            }
        }

        // Update from X-Forwarded-() headers.
        if trusted_proxy_headers.contains(XForwardedHost::name()) {
            if let Some(x_forwarded_host) = req.headers().typed_get::<XForwardedHost>() {
                authority = Some(x_forwarded_host.into())
            }
        }

        if trusted_proxy_headers.contains(XForwardedProto::name()) {
            if let Some(x_forwarded_proto) = req.headers().typed_get::<XForwardedProto>() {
                scheme = x_forwarded_proto.deref().deref().into();
            }
        }

        let mut builder = iri_string::build::Builder::new();

        // Set scheme.
        builder.scheme(scheme.as_str());

        // Set authority.
        if let Some(authority) = authority.as_ref() {
            // Set host name.
            builder.host(authority.hostname());
            // Set port.
            if let Some(port) = authority.port() {
                builder.port(port);
            }
        }

        // Set path.
        builder.path(req.uri().path());

        // Set query.
        if let Some(query) = req.uri().query() {
            builder.query(query);
        }

        debug!("Target uri builder: {:?}", builder);

        // If target uri reconstruction success.
        if let Some(target_uri) = builder.build::<UriAbsoluteStr>().ok().and_then(|built| {
            AbsoluteHttpUri::try_new_from(AsRef::<UriStr>::as_ref(&UriAbsoluteString::from(built)))
                .ok()
        }) {
            info!("Reconstructed target uri: {:?}", target_uri);
            // Attach extension.
            req.extensions_mut().insert(target_uri);
            // Delegate to inner service.
            Either::Left(self.inner.call(req))
        }
        // On resolution failure.
        else {
            error!("Error in reconstructing target uri.");
            // Return error response.
            Either::Right(future::ready(Ok(ApiError::builder(
                StatusCode::BAD_REQUEST,
            )
            .message("Invalid request target")
            .finish()
            .into_hyper_response())))
        }
    }
}
