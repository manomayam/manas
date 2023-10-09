//! I define a basic implementation of [`Pod`](super::Pod).
//!

use std::sync::Arc;

use dyn_problem::Problem;
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use manas_storage::SolidStorage;
use tracing::error;

use crate::pod::Pod;

/// A basic implementation of [`Pod`], that provides only the storage.
#[derive(Debug, Clone)]
pub struct BasicPod<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,
}

impl<Storage: SolidStorage> Pod for BasicPod<Storage> {
    type Storage = Storage;

    #[inline]
    fn storage(&self) -> &Arc<Self::Storage> {
        &self.storage
    }

    fn initialize(&self) -> BoxFuture<'static, Result<(), Problem>> {
        self.storage
            .initialize()
            .inspect_err(|e| {
                error!("Error in initializing the storage. {e}");
            })
            .boxed()
    }
}
