//! I define traits and implementations to
//! define pods and service interfaces to them.
//!

use std::sync::Arc;

use dyn_problem::Problem;
use futures::future::BoxFuture;
use manas_space::{resource::uri::SolidResourceUri, SolidStorageSpace};
use manas_storage::{SolidStorage, SolidStorageExt};

pub mod service;

pub mod impl_;

/// A [`Pod`] provides one storage and optionally any number of other goodies.
pub trait Pod: Send + Sync + 'static {
    /// Type of the storage this pod provides.
    type Storage: SolidStorage;

    /// Get storage of this pod.
    fn storage(&self) -> &Arc<Self::Storage>;

        /// Initialize the pod set.
    fn initialize(&self) -> BoxFuture<'static, Result<(), Problem>>;
}

mod sealed {
    use super::Pod;

    pub trait Sealed {}

    impl<T: Pod> Sealed for T {}
}

/// Extension methods over a [`Pod`].
pub trait PodExt: Pod + sealed::Sealed {
    /// Get id of the pod.
    /// Pod id is same as uri of the pod storage's root resource
    #[inline]
    fn id(&self) -> &SolidResourceUri {
        self.storage().space().root_res_uri()
    }
}

impl<T: Pod> PodExt for T {}
