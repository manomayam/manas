//! I define an implementation of [`RepoContext`] for [`PatchingRepo`](super::PatchingRepo).
//!

use std::{fmt::Debug, sync::Arc};

use manas_repo::{
    context::{LayeredRepoContext, RepoContext},
    service::patcher_resolver::impl_::UnsupportedRepPatcher,
    Repo,
};

use super::{patcher::DirectRepPatcher, MRepo};

/// An implementation of [`RepoContext`] for [`PatchingRepo`](super::PatchingRepo).
#[derive(Debug)]
pub struct PatchingRepoContext<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    /// Inner repo config.
    pub inner: Arc<IR::Context>,

    /// Patcher resolution config.
    pub patcher_resolution_config: Arc<P::ResolutionConfig>,
}

impl<IR, P> RepoContext for PatchingRepoContext<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Repo = MRepo<IR, P>;

    #[inline]
    fn storage_space(&self) -> &Arc<IR::StSpace> {
        self.inner.storage_space()
    }
}

impl<IR, P> LayeredRepoContext for PatchingRepoContext<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type InnerRepo = IR;

    #[inline]
    fn inner(&self) -> &Arc<IR::Context> {
        &self.inner
    }
}
