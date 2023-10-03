//! i provide trait and few implementation helpers for admin pods.
//!

use std::sync::Arc;

use dyn_problem::Problem;
use futures::future::BoxFuture;
use manas_space::resource::uri::SolidResourceUri;

use crate::pod::Pod;

pub mod impl_;

/// A trait for admin pods.
/// Admin pods along with satisfying generic pod interface,
/// provides few following functionalities for the management of
/// pods in a dynamic pod set.
///
/// ## Provisions container
/// A provisions container is a container resource that also
/// executes provisioning of pods as a side effect on posting a
/// provision resource to it.
///
/// ## Storage descriptions.
/// An admin pod maintains storage descriptions of all the
/// provisioned pods.
pub trait AdminPod: Pod {
    /// Type of the member pods.
    type MemberPod: Pod;

    /// Check if admin pod's member's uri namespace contains given uri.
    fn has_in_members_uri_ns(&self, uri: &SolidResourceUri) -> bool;

    /// Resolve the target member pod.
    ///
    /// ## Errors:
    /// Should return following problems on specified cases.
    ///
    /// - [`TARGET_NOT_IN_NAMESPACE`], if request target is not in namespace of the member pods.
    ///
    /// - [`TARGET_IN_UNPROVISIONED_POD_NAMESPACE`], If request target
    /// is in namespace of an unprovisioned member pod.
    fn resolve_target_member_pod(
        &self,
        req_target: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::MemberPod>, Problem>>;

    /// Get the member pod with given id.
    fn get_member_pod(
        &self,
        member_pod_id: &SolidResourceUri,
    ) -> BoxFuture<'static, Result<Arc<Self::MemberPod>, Problem>>;
}
