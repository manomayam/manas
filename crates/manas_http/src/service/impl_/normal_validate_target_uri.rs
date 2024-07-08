//! I define middleware service for ensuring request target uri is normalized.
//!

use std::{
    fmt::Debug,
    task::{Context, Poll},
};

use crate::body::Body;
use futures::future::{self, Either, Ready};
use gdp_rs::{inference_rule::IdentityTransform, predicate::impl_::all_of::IntoPL};
use http::{header::LOCATION, Request, Response, StatusCode};
use tower::Service;

use crate::uri::{
    invariant::{AbsoluteHttpUri, NormalAbsoluteHttpUri},
    predicate::is_normal::IsNormal,
};

/// A middleware [`Service`] that validates if request target uri is normalized.
///
/// It expects reconstructed absolute uri of the resource to
/// be available as [`AbsoluteHttpUri`] typed extension.
/// If uri is a normalized http absolute uri. it delegates
/// request to inner service with [`NormalAbsoluteHttpUri`]
/// typed extension set.
/// If not, it will return redirect response with normalized uri as it's `Location`.
#[derive(Debug, Clone)]
pub struct NormalValidateTargetUri<S>
where
    S: Clone,
{
    inner: S,
}

impl<S> NormalValidateTargetUri<S>
where
    S: Clone,
{
    /// Create a new [`NormalValidateTargetUri`].
    #[inline]
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<Request<Body>> for NormalValidateTargetUri<S>
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
        if req.extensions().get::<NormalAbsoluteHttpUri>().is_some() {
            return Either::Left(self.inner.call(req));
        }

        // Get request target's original uri.
        let target_uri = req
            .extensions()
            .get::<AbsoluteHttpUri>()
            .expect("Normal validation of uri must be done after reconstructing target uri.")
            .clone();

        match target_uri.is_http_normalized() {
            // If target_uri is normal, then set extension, and delegate to inner service.
            true => {
                let resource_uri = target_uri
                    .infer::<IntoPL<_, _>>(IdentityTransform::default())
                    .try_extend_predicate::<IsNormal>()
                    .expect("Must be normal, as checked above");
                req.extensions_mut()
                    .insert::<NormalAbsoluteHttpUri>(resource_uri);
                Either::Left(self.inner.call(req))
            }
            false => {
                let response = Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, target_uri.http_normalized().as_str())
                    .body(Body::empty())
                    .expect("Must be valid");

                Either::Right(future::ready(Ok(response)))
            }
        }
    }
}
