//! I define an implementation of [`PodService`].
//! that serves static storage description resource at configured
//! path.
//!

use std::{convert::Infallible, ops::Deref, sync::Arc, task::Poll};

use dyn_problem::Problem;
use futures::{future::BoxFuture, TryFutureExt};
use http_api_problem::ApiError;
use hyper::{service::Service, Body, Method, Request, Response, StatusCode};
use manas_http::service::{namespaced::NamespacedHttpService, BoxHttpResponseFuture};
use manas_space::{resource::uri::SolidResourceUri, SolidStorageSpace};
use manas_storage::SolidStorageExt;
use rdf_dynsyn::{
    serializer::triples::DynSynTripleSerializerFactory,
    syntax::invariant::triples_serializable::TS_TURTLE,
};
use rdf_utils::model::term::ArcTerm;
use rdf_vocabularies::ns;
use sophia_api::{
    serializer::{Stringifier, TripleSerializer},
    term::Term,
};
use tower::ServiceExt;
use tracing::{error, info};

use crate::pod::{
    service::{PodService, PodServiceFactory},
    Pod,
};

/// An implementation of [`PodService`], that wraps another pod-service,
/// and intercepts requests targeting storage description resource, and serves them.
#[derive(Debug, Clone)]
pub struct StorageDescribingPodService<Inner> {
    /// Inner svc.
    pub inner: Inner,
}

impl<Inner> Service<Request<Body>> for StorageDescribingPodService<Inner>
where
    Inner: PodService + Clone,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[allow(unused_qualifications)]
    #[tracing::instrument(skip_all, name = "StorageDescribingPodService::call")]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let storage = self.inner.pod().storage().clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let res_uri = req
                .extensions()
                .get::<SolidResourceUri>()
                .expect("Must be called after uri normal validity check.");

            // If res_uri is description resource uri, then handle the request.
            if res_uri == storage.space().description_res_uri() {
                info!("Request target is storage description resource.");

                // Only allow HEAD and GET, as the resource is derived.
                if ![Method::GET, Method::HEAD].contains(req.method()) {
                    error!("Storage description resource is currently immutable.");
                    return Ok(ApiError::builder(StatusCode::METHOD_NOT_ALLOWED)
                        .message("Storage description resource is currently immutable.")
                        .finish()
                        .into_hyper_response());
                }

                // Otherwise create description response.
                // TODO must use existing representation stack.
                let statements = vec![[
                    storage
                        .space()
                        .root_res_uri()
                        .deref()
                        .into_term::<ArcTerm>(),
                    ns::rdf::type_.into_term(),
                    ns::pim::Storage.into_term(),
                ]];
                let body = DynSynTripleSerializerFactory::default()
                    .new_stringifier(TS_TURTLE)
                    .serialize_triples(statements.into_iter().map(Result::<_, Infallible>::Ok))
                    .expect("Must be valid.")
                    .as_str()
                    .to_string();

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/turtle")
                    .body(Body::from(body))
                    .expect("Must be valid."))
            }
            // Else, delegate to inner service.
            else {
                ServiceExt::<Request<Body>>::ready(&mut inner)
                    .and_then(|svc| svc.call(req))
                    .await
            }
        })
    }
}

impl<Inner> PodService for StorageDescribingPodService<Inner>
where
    Inner: PodService + Clone,
{
    type Pod = Inner::Pod;

    #[inline]
    fn pod(&self) -> &Arc<Self::Pod> {
        self.inner.pod()
    }
}

impl<Inner> NamespacedHttpService<Body, Body> for StorageDescribingPodService<Inner>
where
    Inner: PodService + Clone,
{
    #[inline]
    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool {
        self.inner.has_in_uri_ns(uri)
    }
}

impl<Inner> Service<()> for StorageDescribingPodService<Inner>
where
    Inner: PodService + Clone,
{
    type Response = bool;

    type Error = Problem;

    type Future = BoxFuture<'static, Result<bool, Problem>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<()>::poll_ready(&mut self.inner, cx)
    }

    #[inline]
    fn call(&mut self, _req: ()) -> Self::Future {
        self.inner.call(())
    }
}

/// A [`StorageDescribingPodServiceFactory`] resolves a [`StorageDescribingPodService`]
/// for each pod.
#[derive(Debug, Clone, Default)]
pub struct StorageDescribingPodServiceFactory<InnerFactory> {
    /// Inner factory.
    pub inner_factory: Arc<InnerFactory>,
}

impl<InnerFactory> PodServiceFactory for StorageDescribingPodServiceFactory<InnerFactory>
where
    InnerFactory: PodServiceFactory,
    InnerFactory::Service: Clone,
{
    type Pod = InnerFactory::Pod;
    type Service = StorageDescribingPodService<InnerFactory::Service>;

    #[inline]
    fn new_service(&self, pod: Arc<InnerFactory::Pod>) -> Self::Service {
        Self::Service {
            inner: self.inner_factory.new_service(pod),
        }
    }
}
