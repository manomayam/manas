use std::{convert::Infallible, marker::PhantomData, task::Poll};

use http::{Request, Response};
use http_uri::invariant::NormalAbsoluteHttpUri;
use tower::Service;
use tracing::info;

use crate::service::{namespaced::NamespacedHttpService, BoxHttpResponseFuture, HttpService};

/// An implementation of [`HttpService`], that routes it's
/// requests to a configured overrider if request target is in
/// it's namespace, or else to inner service.
///
/// It expects reconstructed absolute uri of resource target
/// as [`NormalAbsoluteHttpUri`] typed request extension.
///
#[derive(Debug)]
pub struct OverridingHttpService<ReqBody, ResBody, Inner, Overrider> {
    /// Inner service.
    pub inner: Inner,

    /// Overrider service.
    pub overrider: Overrider,
    _phantom: PhantomData<fn() -> (ReqBody, ResBody)>,
}

impl<ReqBody, ResBody, Inner: Clone, Overrider: Clone> Clone
    for OverridingHttpService<ReqBody, ResBody, Inner, Overrider>
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            overrider: self.overrider.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<ReqBody, ResBody, Inner, Overrider> OverridingHttpService<ReqBody, ResBody, Inner, Overrider>
where
    Inner: HttpService<ReqBody, ResBody>,
    Overrider: NamespacedHttpService<ReqBody, ResBody>,
{
    /// Get new [`OverridingHttpService`] with given inner
    /// http-service, and overrider service.
    #[inline]
    pub fn new(inner: Inner, overrider: Overrider) -> Self {
        Self {
            inner,
            overrider,
            _phantom: PhantomData,
        }
    }
}

impl<ReqBody, ResBody, Inner, Overrider> Service<Request<ReqBody>>
    for OverridingHttpService<ReqBody, ResBody, Inner, Overrider>
where
    Inner: HttpService<ReqBody, ResBody>,
    Overrider: NamespacedHttpService<ReqBody, ResBody>,
{
    type Response = Response<ResBody>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<ResBody>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.inner.poll_ready(cx) {
            Poll::Ready(_) => self.overrider.poll_ready(cx),
            Poll::Pending => Poll::Pending,
        }
    }

    #[tracing::instrument(skip_all, name = "OverridenPodService__call")]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Get res uri.
        let res_uri = req
            .extensions()
            .get::<NormalAbsoluteHttpUri>()
            .expect("Service must be called after handling uri normal validity check.");

        info!("res_uri: {:?}", res_uri);

        // First check if overrider service has the uri in it's target namespace.
        if self.overrider.has_in_uri_ns(res_uri) {
            info!("routed to overrider.");
            self.overrider.call(req)
        } else {
            // Otherwise call inner podset-service.
            info!("routed to inner podset.");
            self.inner.call(req)
        }
    }
}

impl<ReqBody, ResBody, Inner, Overrider> NamespacedHttpService<ReqBody, ResBody>
    for OverridingHttpService<ReqBody, ResBody, Inner, Overrider>
where
    ReqBody: 'static,
    ResBody: 'static,
    Inner: NamespacedHttpService<ReqBody, ResBody> + Clone,
    Overrider: NamespacedHttpService<ReqBody, ResBody> + Clone,
{
    fn has_in_uri_ns(&self, uri: &NormalAbsoluteHttpUri) -> bool {
        // Return true if uri is in namespace of either inner svc, or overrider.
        self.overrider.has_in_uri_ns(uri) || self.inner.has_in_uri_ns(uri)
    }
}
