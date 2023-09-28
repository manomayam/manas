//! I define a basic implementation of [`Pod`](super::Pod).
//!

use std::sync::Arc;

use manas_storage::SolidStorage;

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
}
