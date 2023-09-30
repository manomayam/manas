//! I define an implementation of [`RepoContext`] for [`DelegatingRepo`](super::DelegatingRepo).
//!

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use manas_repo::{
    context::{LayeredRepoContext, RepoContext},
    Repo,
};

use super::MRepo;

/// An implementation of [`RepoContext`] for [`DelegatingRepo`](super::DelegatingRepo).
pub struct DelegatingRepoContext<IR, DLR>
where
    IR: Repo,
{
    /// Inner repo config.
    pub inner: Arc<IR::Context>,

    /// Layer config.
    pub layer_config: Arc<PhantomData<fn(DLR)>>,
}

impl<IR, DLR> Debug for DelegatingRepoContext<IR, DLR>
where
    IR: Repo,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatingRepoContext")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<IR, DLR> RepoContext for DelegatingRepoContext<IR, DLR>
where
    IR: Repo,
    DLR: 'static,
{
    type Repo = MRepo<IR, DLR>;

    #[inline]
    fn storage_space(&self) -> &Arc<IR::StSpace> {
        self.inner.storage_space()
    }
}

impl<IR, DLR> LayeredRepoContext for DelegatingRepoContext<IR, DLR>
where
    IR: Repo,
    DLR: 'static,
{
    type InnerRepo = IR;

    #[inline]
    fn inner(&self) -> &Arc<IR::Context> {
        &self.inner
    }
}
