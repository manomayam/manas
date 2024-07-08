//! I define a basic implementation of [`PodService`].
//!

use std::{convert::Infallible, marker::PhantomData, sync::Arc, task::Poll};

use dyn_problem::Problem;
use futures::future::BoxFuture;
use http::{Request, Response};
use manas_http::{
    body::Body,
    service::{namespaced::NamespacedHttpService, BoxHttpResponseFuture},
};
use manas_space::resource::uri::SolidResourceUri;
use manas_storage::service::{SolidStorageService, SolidStorageServiceFactory};
use tower::Service;

use crate::pod::{
    service::{PodService, PodServiceFactory},
    Pod,
};

/// A basic implementation of [`PodService`].
/// It just serves http interface to pod's storage through
/// configured storage service.
///
#[derive(Debug)]
pub struct BasicPodService<SvcPod, StorageSvc> {
    /// Pod.
    pod: Arc<SvcPod>,

    /// Storage service.
    storage_svc: StorageSvc,
}

impl<SvcPod, StorageSvc: Clone> Clone for BasicPodService<SvcPod, StorageSvc> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            pod: self.pod.clone(),
            storage_svc: self.storage_svc.clone(),
        }
    }
}

impl<SvcPod, StorageSvc> Service<Request<Body>> for BasicPodService<SvcPod, StorageSvc>
where
    SvcPod: Pod,
    StorageSvc: SolidStorageService<Storage = SvcPod::Storage>,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<Request<Body>>::poll_ready(&mut self.storage_svc, cx)
    }

    #[inline]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.storage_svc.call(req)
    }
}

impl<SvcPod, StorageSvc> NamespacedHttpService<Body, Body> for BasicPodService<SvcPod, StorageSvc>
where
    SvcPod: Pod,
    StorageSvc: SolidStorageService<Storage = SvcPod::Storage> + Clone,
{
    #[inline]
    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool {
        self.storage_svc.has_in_uri_ns(uri)
    }
}

impl<SvcPod, StorageSvc> PodService for BasicPodService<SvcPod, StorageSvc>
where
    SvcPod: Pod,
    StorageSvc: SolidStorageService<Storage = SvcPod::Storage> + Clone,
{
    type Pod = SvcPod;

    #[inline]
    fn pod(&self) -> &Arc<Self::Pod> {
        &self.pod
    }
}

impl<SvcPod, StorageSvc> Service<()> for BasicPodService<SvcPod, StorageSvc>
where
    SvcPod: Pod,
    StorageSvc: SolidStorageService<Storage = SvcPod::Storage>,
{
    type Response = bool;

    type Error = Problem;

    type Future = BoxFuture<'static, Result<bool, Problem>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<()>::poll_ready(&mut self.storage_svc, cx)
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "BasicPodService::call")]
    fn call(&mut self, _req: ()) -> Self::Future {
        Service::<()>::call(&mut self.storage_svc, ())
    }
}

/// A [`BasicPodServiceFactory`] resolves a [`BasicPodService`]
/// for each pod.
#[derive(Debug, Default)]
pub struct BasicPodServiceFactory<SvcPod, StorageSvcFactory> {
    /// Storage service factory.
    pub storage_svc_factory: Arc<StorageSvcFactory>,
    _phantom: PhantomData<fn() -> SvcPod>,
}

impl<SvcPod, StorageSvcFactory> Clone for BasicPodServiceFactory<SvcPod, StorageSvcFactory> {
    fn clone(&self) -> Self {
        Self {
            storage_svc_factory: self.storage_svc_factory.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<SvcPod, StorageSvcFactory> BasicPodServiceFactory<SvcPod, StorageSvcFactory> {
    /// Get a new [`BasicPodServiceFactory`] with given storage
    /// service factory.
    #[inline]
    pub fn new(storage_svc_factory: Arc<StorageSvcFactory>) -> Self {
        Self {
            storage_svc_factory,
            _phantom: PhantomData,
        }
    }
}

impl<SvcPod, StorageSvcFactory> PodServiceFactory
    for BasicPodServiceFactory<SvcPod, StorageSvcFactory>
where
    SvcPod: Pod,
    StorageSvcFactory: SolidStorageServiceFactory<Storage = SvcPod::Storage>,
    StorageSvcFactory::Service: Clone,
{
    type Pod = SvcPod;
    type Service = BasicPodService<SvcPod, StorageSvcFactory::Service>;

    #[inline]
    fn new_service(&self, pod: Arc<SvcPod>) -> Self::Service {
        Self::Service {
            storage_svc: self.storage_svc_factory.new_service(pod.storage().clone()),
            pod,
        }
    }
}
