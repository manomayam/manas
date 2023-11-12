//! I define an implementation of [`PodSet`] with a static set of explicitly enumerated members.
//!

use std::{ops::Deref, sync::Arc};

use dyn_problem::Problem;
use futures::future::BoxFuture;
use manas_space::resource::uri::SolidResourceUri;
use regex::RegexSet;

use crate::{
    pod::{Pod, PodExt},
    podset::{PodSet, POD_NOT_IN_SET, TARGET_NOT_IN_NAMESPACE},
};

/// An implementation of [`PodSet`], that is backed by
/// a static set of enumerated pre-provisioned pods.
#[derive(Debug, Clone)]
pub struct StaticPodSet<MPod> {
    /// Enumeration of member pods.
    pods: Vec<Arc<MPod>>,

    /// Uri namespace regex set.
    uri_ns_regex_set: RegexSet,
}

impl<MPod> StaticPodSet<MPod>
where
    MPod: Pod,
{
    /// Get a new [`StaticPodSet`] with given enumeration of pre provisioned pods.
    pub fn new(mut pods: Vec<Arc<MPod>>) -> Self {
        // Sort pods by length of the storage root resource uri.
        pods.sort_by_key(|pod| pod.deref().id().as_str().len());

        // Create podset's uri namespace regex set.
        let uri_ns_regex_set = RegexSet::new(
            pods.iter()
                // Regex to match resource uris in a pod.
                .map(|pod| format!("^{}", regex::escape(pod.id().as_str()))),
        )
        .expect("Must be valid.");

        Self {
            pods,
            uri_ns_regex_set,
        }
    }
}

impl<MPod: Pod> PodSet for StaticPodSet<MPod> {
    type Pod = MPod;

    #[inline]
    fn initialize(&mut self) -> BoxFuture<'static, Result<(), Problem>> {
        Box::pin(async { Ok(()) })
    }

    #[inline]
    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool {
        self.uri_ns_regex_set.is_match(uri.as_str())
    }

    fn resolve_target_pod(
        &self,
        req_target: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::Pod>, Problem>> {
        // Match request target against namespace uri regex set.
        let matches = self.uri_ns_regex_set.matches(req_target.as_str());

        Box::pin(futures::future::ready(
            if let Some(i) = matches.into_iter().next() {
                Ok(self.pods[i].clone())
            } else {
                Err(TARGET_NOT_IN_NAMESPACE.new_problem())
            },
        ))
    }

    fn get_pod(
        &self,
        pod_id: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::Pod>, Problem>> {
        Box::pin(futures::future::ready(
            self.pods
                .iter()
                .find_map(|pod| (pod.id() == pod_id).then_some(pod.clone()))
                .ok_or_else(|| POD_NOT_IN_SET.new_problem()),
        ))
    }
}
