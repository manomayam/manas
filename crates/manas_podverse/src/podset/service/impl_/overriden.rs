//! I define an implementation of [`PodSetService`] that wraps
//! an inner pod set service and an route overrider.
//!

use std::{convert::Infallible, sync::Arc};

use hyper::{Body, Request, Response};
use manas_http::service::{
    impl_::OverridingHttpService, namespaced::NamespacedHttpService, BoxHttpResponseFuture,
};
use manas_space::resource::uri::SolidResourceUri;
use tower::Service;

use crate::podset::service::PodSetService;

/// An implementation of [`PodSetService`] that allows
/// to override certain routes with custom overrider service.
#[derive(Debug, Clone)]
pub struct OverridenPodSetService<Inner, Overrider>
where
    Inner: PodSetService,
    Overrider: NamespacedHttpService<Body, Body>,
{
    svc: OverridingHttpService<Body, Body, Inner, Overrider>,
}

impl<Inner, Overrider> OverridenPodSetService<Inner, Overrider>
where
    Inner: PodSetService,
    Overrider: NamespacedHttpService<Body, Body>,
{
    /// Create a new [`OverridenPodSetService`], with given params.
    #[inline]
    pub fn new(inner: Inner, overrider: Overrider) -> Self {
        Self {
            svc: OverridingHttpService::new(inner, overrider),
        }
    }
}

impl<Inner, Overrider> PodSetService for OverridenPodSetService<Inner, Overrider>
where
    Inner: PodSetService + Clone,
    Overrider: NamespacedHttpService<Body, Body> + Clone,
{
    type SvcPodSet = Inner::SvcPodSet;

    #[inline]
    fn pod_set(&self) -> &Arc<Self::SvcPodSet> {
        self.svc.inner.pod_set()
    }
}

impl<Inner, Overrider> Service<Request<Body>> for OverridenPodSetService<Inner, Overrider>
where
    Inner: PodSetService,
    Overrider: NamespacedHttpService<Body, Body>,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "OverridenPodSetService::call")]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.svc.call(req)
    }
}

impl<Inner, Overrider> NamespacedHttpService<Body, Body>
    for OverridenPodSetService<Inner, Overrider>
where
    Inner: PodSetService + Clone,
    Overrider: NamespacedHttpService<Body, Body> + Clone,
{
    #[inline]
    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool {
        self.svc.has_in_uri_ns(uri)
    }
}
