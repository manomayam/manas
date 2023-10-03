//! I define traits and implementations to
//! define podsets and service interfaces to them.
//!

use std::sync::Arc;

use dyn_problem::{define_anon_problem_types, Problem};
use futures::future::BoxFuture;
use manas_space::resource::uri::SolidResourceUri;

use crate::pod::Pod;

pub mod impl_;
pub mod service;

/// A trait for defining podsets.
/// A podset manages lifecycle and access to a number of pods.
///
pub trait PodSet: Send + Sync + 'static {
    /// Type of pods in this set.
    type Pod: Pod;

    /// Initialize the pod set.
    fn initialize(&mut self) -> BoxFuture<'static, Result<(), Problem>>;

    /// Check if given uri is in uri namespace of the podset..
    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool;

    /// Resolve provisioned pod corresponding to given request target.
    ///
    /// ### Errors:
    ///
    /// Should return following problems on specified cases.
    ///
    /// - [`TARGET_NOT_IN_NAMESPACE`], if request target is not in namespace of the podset.
    ///
    /// - [`TARGET_IN_UNPROVISIONED_POD_NAMESPACE`], If request target
    /// is in namespace of an unprovisioned pod, that belongs to podset's namespace.
    fn resolve_target_pod(
        &self,
        req_target: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::Pod>, Problem>>;

    /// Get provisioned pod with given pod id.
    /// ### Errors:
    ///
    /// Should return following problems on specified cases.
    ///
    /// - [`POD_NOT_IN_SET`], If pod with given id is not in the podset.
    ///
    /// - [`POD_IS_UNPROVISIONED`], If pod with given id is not yet provisioned.
    fn get_pod(
        &self,
        pod_id: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::Pod>, Problem>>;
}

define_anon_problem_types!(
    /// Target not in namespace.
    TARGET_NOT_IN_NAMESPACE: ("Target not in namespace.");

    /// Target in unprovisioned pod namespace.
    TARGET_IN_UNPROVISIONED_POD_NAMESPACE: ("Target in unprovisioned pod namespace.");

    /// Pod not in set.
    POD_NOT_IN_SET: ("Pod not in set.");

    /// Pod is unprovisioned.
    POD_IS_UNPROVISIONED: ("Pod is unprovisioned.");
);
