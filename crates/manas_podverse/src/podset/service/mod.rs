//! I define traits and implementations to
//! define http interface services to podsets.
//!

use std::sync::Arc;

use manas_http::body::Body;
use manas_http::service::namespaced::NamespacedHttpService;

use super::PodSet;

pub mod impl_;

/// A[`PodSetService`] provides http interface to access and manage a [`PodSet`].
///
pub trait PodSetService: NamespacedHttpService<Body, Body> {
    /// Type of the podset.
    type SvcPodSet: PodSet;

    /// Get the podset, which this service serves for.
    fn pod_set(&self) -> &Arc<Self::SvcPodSet>;
}
