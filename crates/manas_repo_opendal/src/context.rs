//! This module implements [`RepoContext`] for ODR.
//!

use std::sync::Arc;

use manas_repo::context::RepoContext;
use opendal::Capability;

use crate::{
    config::ODRConfig,
    object_store::{backend::ODRObjectStoreBackend, object_space::ODRObjectSpace, ODRObjectStore},
    setup::{ODROstBackend, ODRSetup},
    OpendalRepo,
};

/// A struct representing context for an opendal resource store.
#[derive(Debug, Clone)]
pub struct ODRContext<Setup>
where
    Setup: ODRSetup,
{
    /// Object store for this repo.
    pub object_store: ODRObjectStore<Setup::ObjectStoreSetup>,

    /// Configuration for the store.
    pub config: ODRConfig,
}

impl<Setup> RepoContext for ODRContext<Setup>
where
    Setup: ODRSetup,
{
    type Repo = OpendalRepo<Setup>;

    #[inline]
    fn storage_space(&self) -> &Arc<Setup::StSpace> {
        self.object_store.space.assoc_storage_space()
    }
}

impl<Setup> ODRContext<Setup>
where
    Setup: ODRSetup,
{
    /// Get backend capabilities.
    #[inline]
    pub fn backend_caps(&self) -> Capability {
        self.object_store.backend.operator().info().capability()
    }

    /// Create a new [`ODRContext`] from given params.
    #[inline]
    pub fn new(
        storage_space: Arc<Setup::StSpace>,
        object_store_backend: ODROstBackend<Setup>,
        config: ODRConfig,
    ) -> Self {
        Self {
            object_store: ODRObjectStore {
                space: ODRObjectSpace {
                    assoc_storage_space: storage_space,
                },
                backend: object_store_backend,
            },
            config,
        }
    }
}
