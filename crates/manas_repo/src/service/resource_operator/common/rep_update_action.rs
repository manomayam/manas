//! I define [`RepUpdateAction`].
//!

pub use hyper::body::SizeHint;

use crate::Repo;

/// An enum representing update action over a representation.
pub enum RepUpdateAction<R: Repo> {
    /// Set representation.
    SetWith(R::Representation),

    /// Patch with given patcher.
    PatchWith(R::RepPatcher),
}

impl<R: Repo> std::fmt::Debug for RepUpdateAction<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetWith(arg0) => f.debug_tuple("SetWith").field(arg0).finish(),
            Self::PatchWith(_arg0) => f.debug_tuple("PatchWith").finish(),
        }
    }
}

impl<R: Repo> RepUpdateAction<R> {
    /// Map the repo
    pub fn map_repo<R2>(self) -> RepUpdateAction<R2>
    where
        R2: Repo,
        R::Representation: Into<R2::Representation>,
        R::RepPatcher: Into<R2::RepPatcher>,
    {
        match self {
            RepUpdateAction::SetWith(rep) => RepUpdateAction::SetWith(rep.into()),
            RepUpdateAction::PatchWith(patcher) => RepUpdateAction::PatchWith(patcher.into()),
        }
    }
}
