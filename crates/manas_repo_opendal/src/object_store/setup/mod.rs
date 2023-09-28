//! I define trait for odr object store setup.
//!

use std::fmt::Debug;

use manas_space::SolidStorageSpace;

use super::{backend::ODRObjectStoreBackend, object_space::ODRObjectSpaceSetup};

pub mod impl_;

/// A trait for concrete setup of an odr object store.
pub trait ODRObjectStoreSetup: Send + Sync + 'static + Debug {
    /// Type of the associated storage space
    type AssocStSpace: SolidStorageSpace;

    /// Type of the object space setup.
    type ObjectSpaceSetup: ODRObjectSpaceSetup<AssocStSpace = Self::AssocStSpace>;

    /// Type of the object store backend.
    type Backend: ODRObjectStoreBackend;
}
