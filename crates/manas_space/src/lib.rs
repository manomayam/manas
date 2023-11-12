//! This crate provides abstractions for modeling storage
//! spaces confirming to generalized solid protocol.
//! It also provides few implementation helpers.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

pub mod policy;
pub mod resource;

pub mod impl_;

use std::{fmt::Debug, sync::Arc};

use resource::uri::SolidResourceUri;
use webid::WebId;

use self::{policy::aux::AuxPolicy, resource::slot_id::SolidResourceSlotId};

/// Alias for boxed error trait objects.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A solid storage space is a space composed of resources,
/// in which each resource traces it's provenance to
/// the root resource through a unique slot path.
///
/// A `SolidStorageSpace` has a unique root resource, owner, and
/// a description resource.
///
/// ## Resources:
///
/// Each resource in a solid storage space has following
/// properties defined.
///
/// ### Resource kind:
///
/// A resource in a solid storage space is either a container or
/// a non-container.
///
/// @see [`SolidResourceKind`](resource::kind::SolidResourceKind).
///
/// ### Resource uri:
///
/// Each resource in a solid storage space is identified by a
/// normalized absolute http uri.
///
/// @see [`SolidResourceIUri](resource::uri::SolidResourceUri).
///
/// ### Slot link:
/// Except storage root, every resource is linked to another
/// unique host resource through either a containment
/// relation or an auxiliary relation.
///
/// Thus except storage root, every other resource has a unique
/// slot reverse link from it to it's host resource.
///
/// @see [`SlotRelationType`](resource::slot_rel_type::SlotRelationType),
/// [`SlotLink`](resource::slot_link::SlotLink),
/// [`SlotRevLink`](resource::slot_rev_link::SlotRevLink)
///
/// ### Resource slot id:
///
/// A resource's slot id is defined as the product of resource
/// uri and link to it's storage space.
///
/// @see [`SolidResourceSlotId`](resource::slot_id::SolidResourceSlotId).
///
/// ### Resource slot
/// Each resource in a solid storage space gets assigned an
/// immutable [`SolidResourceSlot`](resource::slot::SolidResourceSlot)
/// when it get's created.
///
/// A resource slot's characteristic is product of resource slot
/// id, it's kind and it's slot reverse link.
///
pub trait SolidStorageSpace: Debug + Clone + PartialEq + Eq + Send + Sync + 'static {
    /// Type representing aux policy of this solid storage space.
    type AuxPolicy: AuxPolicy;

    /// Get uri of storage space root container resource.
    fn root_res_uri(&self) -> &SolidResourceUri;

    /// Get uri of storage space description resource.
    fn description_res_uri(&self) -> &SolidResourceUri;

    /// Get uri of storage space owner.
    fn owner_id(&self) -> &WebId;
}

/// Type alias for known aux rel type of [`SolidStorageSpace`].
pub type SpcKnownAuxRelType<Space> =
    <<Space as SolidStorageSpace>::AuxPolicy as AuxPolicy>::KnownAuxRelType;

/// A relative storage space, with given base resource
/// that belongs to source space as it's root resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeSolidStorageSpace<SourceSpace: SolidStorageSpace> {
    /// Slot id of base resource.
    pub base_res_slot_id: SolidResourceSlotId<SourceSpace>,
}

impl<SourceSpace: SolidStorageSpace> SolidStorageSpace for RelativeSolidStorageSpace<SourceSpace> {
    type AuxPolicy = SourceSpace::AuxPolicy;

    #[inline]
    fn root_res_uri(&self) -> &SolidResourceUri {
        &self.base_res_slot_id.uri
    }

    /// TODO should be different uri.
    #[inline]
    fn description_res_uri(&self) -> &SolidResourceUri {
        self.base_res_slot_id.space.description_res_uri()
    }

    #[inline]
    fn owner_id(&self) -> &WebId {
        self.base_res_slot_id.space.owner_id()
    }
}

impl<BaseSpace: SolidStorageSpace> RelativeSolidStorageSpace<BaseSpace> {
    /// Get base space of this relative space.
    #[inline]
    pub fn base_space(&self) -> &Arc<BaseSpace> {
        &self.base_res_slot_id.space
    }
}

#[cfg(feature = "test-utils")]
/// A module with utils to mock [`SolidStorageSpace`].
pub mod mock {
    use super::{impl_::BasicSolidStorageSpace, policy::aux::mock::MockAuxPolicy, *};

    /// Type alias for mock storage space.
    pub type MockSolidStorageSpace<const MAX_AUX_LINKS: usize = 0> =
        BasicSolidStorageSpace<MockAuxPolicy<MAX_AUX_LINKS>>;

    impl<const MAX_AUX_LINKS: usize> MockSolidStorageSpace<MAX_AUX_LINKS> {
        /// Create a mock storage space from given root resource uri.
        pub fn new_from_valid_root_uri_str(root_res_uri_str: &str) -> Self {
            let root_res_uri = SolidResourceUri::try_new_from(root_res_uri_str)
                .expect("Claimed valid root res uri str");

            let mock_owner_id =
                WebId::try_from(format!("{}#owner", root_res_uri_str).as_str()).unwrap();

            BasicSolidStorageSpace::new(root_res_uri.clone(), root_res_uri, mock_owner_id)
        }
    }
}
