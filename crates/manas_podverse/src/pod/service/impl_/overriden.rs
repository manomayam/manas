//! I define an implementation of [`PodService`] that wraps
//! an inner pod service and an route overrider.
//!

use std::{convert::Infallible, sync::Arc};

use dyn_problem::{ProbFuture, Problem};
use http::{Request, Response};
use manas_http::{
    body::Body,
    service::{
        impl_::OverridingHttpService, namespaced::NamespacedHttpService, BoxHttpResponseFuture,
    },
};
use manas_space::resource::uri::SolidResourceUri;
use tower::Service;

use crate::pod::service::{PodService, PodServiceFactory};

/// An implementation of [`PodService`] that allows
/// to override certain routes with custom overrider service.
#[derive(Debug, Clone)]
pub struct OverridenPodService<Inner, Overrider>
where
    Inner: PodService,
    Overrider: NamespacedHttpService<Body, Body>,
{
    svc: OverridingHttpService<Body, Body, Inner, Overrider>,
}

impl<Inner, Overrider> OverridenPodService<Inner, Overrider>
where
    Inner: PodService,
    Overrider: NamespacedHttpService<Body, Body>,
{
    /// Create a new [`OverridenPodService`], with given params.
    #[inline]
    pub fn new(inner: Inner, overrider: Overrider) -> Self {
        Self {
            svc: OverridingHttpService::new(inner, overrider),
        }
    }
}

impl<Inner, Overrider> PodService for OverridenPodService<Inner, Overrider>
where
    Inner: PodService + Clone,
    Overrider: NamespacedHttpService<Body, Body> + Clone,
{
    type Pod = Inner::Pod;

    #[inline]
    fn pod(&self) -> &Arc<Self::Pod> {
        self.svc.inner.pod()
    }
}

impl<Inner, Overrider> Service<()> for OverridenPodService<Inner, Overrider>
where
    Inner: PodService + Clone,
    Overrider: NamespacedHttpService<Body, Body> + Clone,
{
    type Response = bool;

    type Error = Problem;

    type Future = ProbFuture<'static, bool>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Service::<()>::poll_ready(&mut self.svc.inner, cx)
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "OverridenPodService::call")]
    fn call(&mut self, req: ()) -> Self::Future {
        self.svc.inner.call(req)
    }
}

impl<Inner, Overrider> Service<Request<Body>> for OverridenPodService<Inner, Overrider>
where
    Inner: PodService,
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
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.svc.call(req)
    }
}

impl<Inner, Overrider> NamespacedHttpService<Body, Body> for OverridenPodService<Inner, Overrider>
where
    Inner: PodService + Clone,
    Overrider: NamespacedHttpService<Body, Body> + Clone,
{
    #[inline]
    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool {
        self.svc.has_in_uri_ns(uri)
    }
}

/// A [`OverridenPodServiceFactory`] resolves a [`OverridenPodService`]
/// for each pod.
#[derive(Debug, Clone)]
pub struct OverridenPodServiceFactory<InnerFactory, Overrider> {
    /// Inner factory.
    pub inner_factory: Arc<InnerFactory>,

    /// Overrider service.
    /// TODO MUST be a factory.
    pub overrider: Arc<Overrider>,
}

impl<InnerFactory, Overrider> PodServiceFactory
    for OverridenPodServiceFactory<InnerFactory, Overrider>
where
    InnerFactory: PodServiceFactory,
    InnerFactory::Service: Clone,
    Overrider: NamespacedHttpService<Body, Body> + Clone,
{
    type Pod = InnerFactory::Pod;
    type Service = OverridenPodService<InnerFactory::Service, Overrider>;

    #[inline]
    fn new_service(&self, pod: Arc<InnerFactory::Pod>) -> Self::Service {
        Self::Service::new(
            self.inner_factory.new_service(pod),
            self.overrider.as_ref().clone(),
        )
    }
}
