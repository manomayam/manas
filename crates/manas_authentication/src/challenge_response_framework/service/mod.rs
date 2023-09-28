//! I define an implementation of [`HttpService`]
//! that performs authentication before delegating to inner service.
//!

use std::{convert::Infallible, fmt::Debug, marker::PhantomData, sync::Arc, task::Poll};

use either::Either;
use futures::TryFutureExt;
use headers::HeaderMapExt;
use http::{header::AUTHORIZATION, Method, Request, Response, StatusCode};
use http_uri::invariant::AbsoluteHttpUri;
use manas_http::service::{BoxHttpResponseFuture, HttpService};
use tower::{Layer, Service, ServiceExt};
use tracing::{error, info};

use super::scheme::CRAuthenticationScheme;
use crate::common::req_authenticator::RequestAuthenticator;

///  implementation of [`HttpService`]
/// that performs challenge-response based authentication before delegating to inner service.
#[derive(Debug)]
pub struct HttpCRAuthenticationService<Inner, Scheme, ResBody, Authenticator> {
    /// Inner service.
    inner: Inner,

    /// Authentication scheme.
    scheme: Scheme,

    /// Methods on which authentication is not optional.
    required_on: Arc<Vec<Method>>,

    _phantom: PhantomData<fn(Authenticator, ResBody)>,
}

impl<Inner: Clone, Scheme: Clone, ResBody, Authenticator> Clone
    for HttpCRAuthenticationService<Inner, Scheme, ResBody, Authenticator>
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            scheme: self.scheme.clone(),
            required_on: self.required_on.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<Inner, Scheme, ReqBody, ResBody, Authenticator> Service<Request<ReqBody>>
    for HttpCRAuthenticationService<Inner, Scheme, ResBody, Authenticator>
where
    ReqBody: Send + 'static,
    ResBody: Default,
    Inner: HttpService<ReqBody, ResBody> + Clone,
    Scheme: CRAuthenticationScheme,
    Authenticator: RequestAuthenticator<Credentials = Scheme::Credentials>,
{
    type Response = Response<ResBody>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<ResBody>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "HttpCRAuthenticationService::call")]
    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Get request uri.
        let req_uri: &AbsoluteHttpUri = req.extensions().get().expect(
            "CRAuthenticationService must be called after configuring resource absolute uri.",
        );

        // If there is authorization header, or authorization
        // is non-optional on the method
        // Then creds/challenges should be resolved.
        let should_resolve =
            req.headers().get(AUTHORIZATION).is_some() || self.required_on.contains(req.method());

        // Resolve authentication.
        let auth_resln_fut = if should_resolve {
            self.scheme
                .resolve_or_challenge(req_uri, req.method(), req.headers())
        } else {
            // Continue with default creds.
            Box::pin(async { Ok(Default::default()) })
        };

        let mut inner_svc = self.inner.clone();

        Box::pin(async move {
            match auth_resln_fut.await {
                // On resolution success.
                Ok(credentials) => {
                    // Authenticate the request.
                    req = Authenticator::authenticated(req, credentials);

                    // And delegate further handling to inner service.
                    inner_svc.ready().and_then(|svc| svc.call(req)).await
                }

                // On challenge.
                Err(Either::Left(challenge)) => {
                    info!("Challenge resolved: {:?}", challenge);
                    // Return 401 response.
                    let mut builder = Response::builder().status(StatusCode::UNAUTHORIZED);

                    let headers = builder.headers_mut().expect("Must be some");

                    // Extend with optional headers first.
                    headers.extend(challenge.ext_headers);

                    // Insert www-authenticate.
                    headers.typed_insert(challenge.www_authenticate);

                    Ok(builder
                        .body(Default::default())
                        .expect("Must be valid response."))
                }

                // On resolution failure.
                Err(Either::Right(_)) => {
                    error!("Unknown error in resolving authentication credentials.");
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Default::default())
                        .expect("Must be valid response."))
                }
            }
        })
    }
}

impl<Inner, Scheme, ResBody, Authenticator>
    HttpCRAuthenticationService<Inner, Scheme, ResBody, Authenticator>
{
    /// Create a new [`HttpCRAuthenticationService`] with given params.
    #[inline]
    pub fn new(inner_svc: Inner, auth_scheme: Scheme, required_on: Arc<Vec<Method>>) -> Self {
        Self {
            inner: inner_svc,
            scheme: auth_scheme,
            required_on,
            _phantom: PhantomData,
        }
    }
}

/// A layer that wraps [`HttpCRAuthenticationService`] over inner service.
#[derive(Debug)]
#[allow(clippy::type_complexity)]
pub struct HttpCRAuthenticationLayer<Inner, Scheme, ResBody, Authenticator> {
    /// Authentication scheme.
    scheme: Scheme,

    /// Methods on which authentication is not-optional.
    required_on: Arc<Vec<Method>>,
    _phantom: PhantomData<fn() -> (Inner, ResBody, Authenticator)>,
}

impl<Inner: Clone, Scheme: Clone, ResBody, Authenticator> Clone
    for HttpCRAuthenticationLayer<Inner, Scheme, ResBody, Authenticator>
{
    fn clone(&self) -> Self {
        Self {
            scheme: self.scheme.clone(),
            required_on: self.required_on.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<Inner, Scheme, ResBody, Authenticator>
    HttpCRAuthenticationLayer<Inner, Scheme, ResBody, Authenticator>
{
    /// Create a new [`HttpCRAuthenticationLayer`] with given params.
    pub fn new(auth_scheme: Scheme, required_on: Arc<Vec<Method>>) -> Self {
        Self {
            scheme: auth_scheme,
            required_on,
            _phantom: PhantomData,
        }
    }
}

impl<Inner, Scheme, ResBody, Authenticator> Layer<Inner>
    for HttpCRAuthenticationLayer<Inner, Scheme, ResBody, Authenticator>
where
    Scheme: Clone,
{
    type Service = HttpCRAuthenticationService<Inner, Scheme, ResBody, Authenticator>;

    fn layer(&self, inner: Inner) -> Self::Service {
        HttpCRAuthenticationService::new(inner, self.scheme.clone(), self.required_on.clone())
    }
}
