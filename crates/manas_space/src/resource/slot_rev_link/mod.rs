//! I define [`SlotRevLink`].

use super::{slot_rel_type::SlotRelationType, uri::SolidResourceUri};
use crate::{SolidStorageSpace, SpcKnownAuxRelType};

/// A struct representing a slot reverse link from a
/// resource to it's immediate host resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRevLink<Space>
where
    Space: SolidStorageSpace,
{
    /// Target of link.
    pub target: SolidResourceUri,

    /// Reverse link rel type.
    pub rev_rel_type: SlotRelationType<SpcKnownAuxRelType<Space>>,
}
