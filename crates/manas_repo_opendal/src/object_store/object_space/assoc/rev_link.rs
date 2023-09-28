//! I define [`AssocRevLink`].
//!

use manas_space::resource::slot_id::SolidResourceSlotId;

use super::rel_type::AssocRelType;
use crate::object_store::object_space::ODRObjectSpaceSetup;

/// A struct representing association reverse link
/// from an object in odr object space to target resource in
/// associated storage space.
#[derive(Debug, Clone)]
pub struct AssocRevLink<ObjSpaceSetup>
where
    ObjSpaceSetup: ODRObjectSpaceSetup,
{
    /// Target resource slot id of this association reverse link.
    pub target: SolidResourceSlotId<ObjSpaceSetup::AssocStSpace>,

    /// Reverse link rel type.
    pub rev_rel_type: AssocRelType,
}

impl<OSSetup: PartialEq> PartialEq for AssocRevLink<OSSetup>
where
    OSSetup: ODRObjectSpaceSetup,
{
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.rev_rel_type == other.rev_rel_type
    }
}
