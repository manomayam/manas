//! I provide an implementation of [`PodSet`] that is dynamic
//! over it's member pods.
//!

use std::sync::Arc;

use dyn_problem::Problem;
use futures::{future::BoxFuture, TryFutureExt};
use manas_space::resource::uri::SolidResourceUri;
use tracing::error;

use crate::{pod::Pod, podset::PodSet};

use self::admin_pod::AdminPod;

pub mod admin_pod;

/// An implementation of [`PodSet`] that is dynamic
/// over it's member pods.
#[derive(Debug, Clone)]
pub struct DynamicPodSet<AdmPod> {
    /// Admin pod.
    admin_pod: Arc<AdmPod>,
}

impl<AdmPod: AdminPod> DynamicPodSet<AdmPod> {
    /// Create a new [`DynamicPodSet`] from given admin pod..
    #[inline]
    pub fn new(admin_pod: Arc<AdmPod>) -> Self {
        Self { admin_pod }
    }
}

impl<AdmPod: AdminPod> PodSet for DynamicPodSet<AdmPod> {
    type Pod = AdmPod::MemberPod;

    fn initialize(&mut self) -> BoxFuture<'static, Result<(), Problem>> {
        Box::pin(self.admin_pod.initialize().inspect_err(|e| {
            error!("Error in initializing the admin pod. {e}");
        }))
    }

    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool {
        todo!()
    }

    fn resolve_target_pod(
        &self,
        req_target: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::Pod>, Problem>> {
        todo!()
    }

    fn get_pod(
        &self,
        pod_id: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::Pod>, Problem>> {
        todo!()
    }
}
