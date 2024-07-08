//! I define traits and implementations to
//! define pod-services.
//!

use std::sync::Arc;

use dyn_problem::{ProbFuture, Problem};
use manas_http::body::Body;
use manas_http::service::namespaced::NamespacedHttpService;
use tower::Service;

use super::Pod;

pub mod impl_;

/// A [`PodService`] provides http interface service to a pod.
pub trait PodService: NamespacedHttpService<Body, Body> + PodInitializer {
    /// Type of the pod, this service provides interface to.
    type Pod: Pod;

    /// Get the pod of this service.
    fn pod(&self) -> &Arc<Self::Pod>;
}

/// A contract trait for pod initializer.
///
/// Service must initialize storage provided by the pod.
/// Service must be idempotent.
/// It should return Ok(false), if it is already initialized.
pub trait PodInitializer:
    Service<(), Response = bool, Error = Problem, Future = ProbFuture<'static, bool>>
{
}

impl<S> PodInitializer for S where
    S: Service<(), Response = bool, Error = Problem, Future = ProbFuture<'static, bool>>
{
}

/// A [`PodServiceFactory`] resolves a pod service for each pod..
pub trait PodServiceFactory: Clone + Send + Sync + 'static + Unpin {
    /// Type of pods.
    type Pod: Pod;

    /// Type of pod services.
    type Service: PodService;

    /// Get a new pod service for given pod.
    fn new_service(&self, pod: Arc<Self::Pod>) -> Self::Service;
}
