//! I define [`SolidResourceSlotId`].

use std::sync::Arc;

use super::uri::SolidResourceUri;
use crate::{RelativeSolidStorageSpace, SolidStorageSpace};

/// A [`SolidResourceSlotId`] is a product of resource uri, and
/// a link to the storage space it is part of.
#[derive(Clone, PartialEq, Eq)]
pub struct SolidResourceSlotId<Space>
where
    Space: SolidStorageSpace,
{
    /// Provenience storage space of this resource.
    pub space: Arc<Space>,

    /// Uri of the resource.
    pub uri: SolidResourceUri,
}

impl<Space> std::fmt::Debug for SolidResourceSlotId<Space>
where
    Space: SolidStorageSpace,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceID(<{}> | {:?})", self.uri.as_str(), &self.space)
    }
}

impl<Space> std::fmt::Display for SolidResourceSlotId<Space>
where
    Space: SolidStorageSpace,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<Space> SolidResourceSlotId<Space>
where
    Space: SolidStorageSpace,
{
    /// Get a new [`SolidResourceSlotId`] from given params.
    #[inline]
    pub fn new(space: Arc<Space>, uri: SolidResourceUri) -> Self {
        Self { space, uri }
    }
}

/// Type alias for resource slot id in a relative space.
pub type RelativeResourceSlotId<Space> = SolidResourceSlotId<RelativeSolidStorageSpace<Space>>;

impl<Space> RelativeResourceSlotId<Space>
where
    Space: SolidStorageSpace,
{
    /// Convert this resource id into resource id in absolute
    /// space.
    #[inline]
    pub fn into_absolute(self) -> SolidResourceSlotId<Space> {
        SolidResourceSlotId {
            uri: self.uri,
            space: self.space.base_space().clone(),
        }
    }
}

impl<Space> SolidResourceSlotId<Space>
where
    Space: SolidStorageSpace,
{
    /// Get id of the root slot in a space.
    #[inline]
    pub fn root_slot_id(space: Arc<Space>) -> Self {
        Self {
            uri: space.root_res_uri().clone(),
            space,
        }
    }

    /// Check if slot id is root slot id.
    #[inline]
    pub fn is_root_slot_id(&self) -> bool {
        self.space.root_res_uri() == &self.uri
    }
}

#[cfg(feature = "test-utils")]
/// I define utilities to easily mock [`SolidResourceSlotId`].
pub mod mock {
    use super::*;
    use crate::mock::MockSolidStorageSpace;

    /// Type of resource slot ids in [`MockSolidStorageSpace`]
    pub type MockSpaceResourceSlotId<const MAX_AUX_LINKS: usize> =
        SolidResourceSlotId<MockSolidStorageSpace<MAX_AUX_LINKS>>;

    impl<const MAX_AUX_LINKS: usize> MockSpaceResourceSlotId<MAX_AUX_LINKS> {
        /// Get new [`SolidResourceSlotId`] from given parts.
        /// Method will panic, if supplied values are not valid encoded.
        pub fn new_from_valid_parts(storage_root_uri_str: &str, res_uri_str: &str) -> Self {
            SolidResourceSlotId {
                space: Arc::new(MockSolidStorageSpace::new_from_valid_root_uri_str(
                    storage_root_uri_str,
                )),
                uri: SolidResourceUri::try_new_from(res_uri_str)
                    .expect("Claimed valid slot id parts"),
            }
        }
    }
}
