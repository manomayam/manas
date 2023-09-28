//! I define a basic implementation of
//! [`PodSetService`].
//!

use std::{convert::Infallible, sync::Arc, task::Poll};

use hyper::{Body, Request, Response, StatusCode};
use manas_http::{
    service::{namespaced::NamespacedHttpService, BoxHttpResponseFuture},
    uri::invariant::NormalAbsoluteHttpUri,
};
use tower::{Service, ServiceExt};
use tracing::{error, info, instrument};

use crate::{
    pod::{service::PodServiceFactory, PodExt},
    podset::{
        service::PodSetService, PodSet, TARGET_IN_UNPROVISIONED_POD_NAMESPACE,
        TARGET_NOT_IN_NAMESPACE,
    },
};

/// A basic implementation of [`PodSetService`],
/// that routes requests to services of it's managed pods.
///
#[derive(Debug)]
pub struct BasicPodSetService<SvcPodSet, PodSvcFactory> {
    /// Pod set.
    pub pod_set: Arc<SvcPodSet>,

    /// Pod service factory.
    pub pod_service_factory: Arc<PodSvcFactory>,
}

impl<SvcPodSet, PodSvcFactory> Clone for BasicPodSetService<SvcPodSet, PodSvcFactory> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            pod_set: self.pod_set.clone(),
            pod_service_factory: self.pod_service_factory.clone(),
        }
    }
}

impl<SvcPodSet, PodSvcFactory> Service<Request<Body>>
    for BasicPodSetService<SvcPodSet, PodSvcFactory>
where
    SvcPodSet: PodSet,
    PodSvcFactory: PodServiceFactory<Pod = SvcPodSet::Pod>,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[instrument(skip_all, name = "BasicPodSetService::call")]
    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let pod_set = self.pod_set.clone();
        let pod_svc_factory = self.pod_service_factory.clone();

        Box::pin(async move {
            // Get res uri.
            let res_uri = req.extensions().get::<NormalAbsoluteHttpUri>().expect(
                "BasicPodSetService must be called after handling uri normal validity check.",
            );

            // Resolve target pod.
            match pod_set.resolve_target_pod(res_uri).await {
                Ok(pod) => {
                    info!("Pod resolved for request target. pod: {:?}", pod.id());
                    // Delegate handling to pod service.
                    pod_svc_factory.new_service(pod).oneshot(req).await
                }
                Err(e) => {
                    // Resolve status code.
                    let status = if [
                        &TARGET_NOT_IN_NAMESPACE,
                        &TARGET_IN_UNPROVISIONED_POD_NAMESPACE,
                    ]
                    .iter()
                    .any(|t| t.is_type_of(&e))
                    {
                        error!(
                            "No provisioned pod resolved for request target. Error:\n {}",
                            e
                        );
                        // Return 404
                        StatusCode::NOT_FOUND
                    } else {
                        error!("Unknown error in resolving target pod. Error:\n {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    };

                    Ok(Response::builder()
                        .status(status)
                        .body(Body::empty())
                        .expect("Must be valid hyper response."))
                }
            }
        })
    }
}

impl<SvcPodSet, PodSvcFactory> PodSetService for BasicPodSetService<SvcPodSet, PodSvcFactory>
where
    SvcPodSet: PodSet,
    PodSvcFactory: PodServiceFactory<Pod = SvcPodSet::Pod>,
{
    type SvcPodSet = SvcPodSet;

    #[inline]
    fn pod_set(&self) -> &Arc<Self::SvcPodSet> {
        &self.pod_set
    }
}

impl<SvcPodSet, PodSvcFactory> NamespacedHttpService<Body, Body>
    for BasicPodSetService<SvcPodSet, PodSvcFactory>
where
    SvcPodSet: PodSet,
    PodSvcFactory: PodServiceFactory<Pod = SvcPodSet::Pod>,
{
    fn has_in_uri_ns(&self, uri: &NormalAbsoluteHttpUri) -> bool {
        // Only serves pod resources.
        self.pod_set.has_in_uri_ns(uri)
    }
}
