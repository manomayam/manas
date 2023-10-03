//! I define types and provide few implementations for pod templates.
//!

use std::{borrow::Borrow, fmt::Debug, hash::Hash};

use dyn_problem::ProbFuture;
use manas_space::resource::uri::SolidResourceUri;

use crate::pod::Pod;

/// A trait for pod templates.
pub trait PodTemplate: Debug + Send + Sync + 'static {
    /// Type of pods this template renders.
    type RenderedPod: Pod;

    /// Type of the pod key.
    /// There is a one to one bijection from all instances of
    /// this type, to pods that can be resolved by this template.
    type PodKey: Debug + Send + Sync + 'static + PartialEq + Eq + Hash + Clone + Borrow<str>;

    /// Check if given uri is in namespace of pod template's.
    fn has_in_uri_ns(&self, uri: &SolidResourceUri) -> bool;

    /// Resolve target pod key.
    fn resolve_target_pod_key(&self, req_target: &SolidResourceUri) -> Option<Self::PodKey>;

    /// Try to render the pod corresponding to given pod key.
    fn render(&self, key: &Self::PodKey) -> ProbFuture<'static, Self::RenderedPod>;
}
