//! I define [`AssocLink`].
//!

use super::rel_type::AssocRelType;
use crate::object_store::{object_id::ODRObjectId, object_space::ODRObjectSpaceSetup};

/// A struct representing association link
/// from a resource in associated storage space to target object in an odr object space.
#[derive(Debug, Clone)]
pub struct AssocLink<'obj_id, ObjSpaceSetup>
where
    ObjSpaceSetup: ODRObjectSpaceSetup,
{
    /// Id of target odr object.
    pub target: ODRObjectId<'obj_id, ObjSpaceSetup>,

    /// Link rel type.
    pub rel_type: AssocRelType,
}

impl<'obj_id, OSSetup> PartialEq for AssocLink<'obj_id, OSSetup>
where
    OSSetup: ODRObjectSpaceSetup,
{
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target && self.rel_type == other.rel_type
    }
}

impl<'obj_id, ObjSpaceSetup> Eq for AssocLink<'obj_id, ObjSpaceSetup> where
    ObjSpaceSetup: ODRObjectSpaceSetup
{
}
