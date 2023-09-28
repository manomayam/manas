//! I define [`ODRObjectId`].
//!

use self::normal_rootless_uri_path::NormalRootlessUriPath;
use super::object_space::{ODRObjectSpace, ODRObjectSpaceSetup};

pub mod normal_rootless_uri_path;

/// An [`ODRObjectId`] is a product of reference to it's object space, and it's path relative to space root.
///
/// An object id's `root_relative_path` is independent of storage backend and it's naming constraints.
/// It is always guaranteed to be valid, normalized, rootless uri path.
pub struct ODRObjectId<'path, ObjSpaceSetup: ODRObjectSpaceSetup> {
    /// Shared ref to object space, the object is part of.
    pub space: ODRObjectSpace<ObjSpaceSetup>,

    /// Root relative path of object.
    pub root_relative_path: NormalRootlessUriPath<'path>,
}

impl<'path, OSSetup: ODRObjectSpaceSetup> PartialEq for ODRObjectId<'path, OSSetup> {
    fn eq(&self, other: &Self) -> bool {
        self.space == other.space && self.root_relative_path == other.root_relative_path
    }
}

impl<'path, OSSetup: ODRObjectSpaceSetup> Eq for ODRObjectId<'path, OSSetup> {}

impl<'path, OSSetup> std::fmt::Debug for ODRObjectId<'path, OSSetup>
where
    OSSetup: ODRObjectSpaceSetup,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ODRObjectId(<{}> | AsSp<{:?}>)",
            self.root_relative_path.as_ref(),
            self.space.assoc_storage_space.as_ref()
        )
    }
}

impl<'path, OSSetup> Clone for ODRObjectId<'path, OSSetup>
where
    OSSetup: ODRObjectSpaceSetup,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            space: self.space.clone(),
            root_relative_path: self.root_relative_path.clone(),
        }
    }
}

impl<'path, OSSetup> ODRObjectId<'path, OSSetup>
where
    OSSetup: ODRObjectSpaceSetup,
{
    /// Get an identical but borrowed version of [`ODRObjectId`].
    #[inline]
    pub fn to_borrowed(&self) -> ODRObjectId<'_, OSSetup> {
        ODRObjectId {
            space: self.space.clone(),
            root_relative_path: self.root_relative_path.to_borrowed(),
        }
    }

    /// Convert into owned [`ODRObjectId`].
    #[inline]
    pub fn into_owned(self) -> ODRObjectId<'static, OSSetup> {
        ODRObjectId {
            space: self.space.clone(),
            root_relative_path: self.root_relative_path.into_owned(),
        }
    }
}
