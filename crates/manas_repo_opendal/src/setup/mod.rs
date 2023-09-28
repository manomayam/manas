//! I define [`ODRSetup`] trait for declaring concrete setup for
//! an opendal repo.
//!

pub mod aux_rep_policy;

use std::fmt::Debug;

use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_space::{
    resource::state::{invariant::RepresentedSolidResourceState, SolidResourceState},
    SolidStorageSpace,
};

use self::aux_rep_policy::ODRAuxResourcePolicy;
use crate::object_store::{object_space::ODRObjectSpaceSetup, ODRObjectStoreSetup};

/// Set up for an opendal repo.
pub trait ODRSetup: Debug + 'static + Send + Sync + Clone + Unpin {
    /// Type of storage space of the repo.
    type StSpace: SolidStorageSpace;

    /// Type of odr object store setup.
    type ObjectStoreSetup: ODRObjectStoreSetup<AssocStSpace = Self::StSpace>;

    /// Type of odr aux resource policy.
    type AuxResourcePolicy: ODRAuxResourcePolicy<StSpace = Self::StSpace>;
}

/// Alias for type of storage space of ODR with given setup.
pub type ODRStSpace<Setup> = <Setup as ODRSetup>::StSpace;

/// Type alias for resource state conveyed by ODR.
pub type ODRResourceState<Setup> =
    SolidResourceState<<Setup as ODRSetup>::StSpace, BinaryRepresentation>;

/// Type alias for represented resource state conveyed by ODR.
pub type ODRRepresentedResourceState<Setup> =
    RepresentedSolidResourceState<<Setup as ODRSetup>::StSpace, BinaryRepresentation>;

/// Type of object store backend.
pub type ODROstBackend<Setup> =
    <<Setup as ODRSetup>::ObjectStoreSetup as ODRObjectStoreSetup>::Backend;

/// Type alias for resource semantic slot encoding scheme of odr.
pub type ODRSemSlotES<Setup> = <<<Setup as ODRSetup>::ObjectStoreSetup as ODRObjectStoreSetup>::ObjectSpaceSetup as ODRObjectSpaceSetup>::AssocStSemSlotES;

// /// I define few utils to mock with [`ODRSetup`].
// #[cfg(feature = "test-utils")]
// pub mod mock {
//     use manas_repo::service::resource_operator::common::rep_patcher::impl_::solid_insert_delete::SolidInsertDeletePatcher;
//     use manas_space::mock::MockSolidStorageSpace;

//     use crate::object_store::mock::MockODRObjectStore;

//     use super::{
//         super::service::patcher_resolver::default::DefaultODRRepPatcherResolver,
//         aux_rep_policy::mock::MockODRAuxResourcePolicy,
//         rep_validation_policy::impl_::default::DefaultODRRepValidationPolicy, *,
//     };

//     /// A mock implementation of [`ODRSetup`].
//     #[derive(Debug, Clone)]
//     pub struct MockODRSetup<const MAX_AUX_LINKS: usize = 0> {}

//     impl<const MAX_AUX_LINKS: usize> ODRSetup for MockODRSetup<MAX_AUX_LINKS> {
//         type StSpace = MockSolidStorageSpace<MAX_AUX_LINKS>;

//         type ObjectStore = MockODRObjectStore<MAX_AUX_LINKS>;

//         type AuxResourcePolicy = MockODRAuxResourcePolicy<MAX_AUX_LINKS>;

//         type UserSuppliedRepValidationPolicy = DefaultODRRepValidationPolicy;

//         type RepPatcher = SolidInsertDeletePatcher;

//         type RepPatcherResolver = DefaultODRRepPatcherResolver<Self>;
//     }
// }
