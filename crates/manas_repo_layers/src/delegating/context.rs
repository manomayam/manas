//! I define an implementation of [`RepoContext`] for [`DelegatingRepo`](super::DelegatingRepo).
//!

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use manas_repo::{
    context::{LayeredRepoContext, RepoContext},
    Repo,
};

use super::MRepo;

/// An implementation of [`RepoContext`] for [`DelegatingRepo`](super::DelegatingRepo).
pub struct DelegatingRepoContext<IR, V>
where
    IR: Repo,
{
    /// Inner repo config.
    pub inner: Arc<IR::Context>,

    _phantom: PhantomData<fn(V)>,
}

impl<IR, V> Debug for DelegatingRepoContext<IR, V>
where
    IR: Repo,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatingRepoContext")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<IR, V> RepoContext for DelegatingRepoContext<IR, V>
where
    IR: Repo,
    V: 'static,
{
    type Repo = MRepo<IR, V>;

    #[inline]
    fn storage_space(&self) -> &Arc<IR::StSpace> {
        self.inner.storage_space()
    }
}

impl<IR, V> LayeredRepoContext for DelegatingRepoContext<IR, V>
where
    IR: Repo,
    V: 'static,
{
    type InnerRepo = IR;

    #[inline]
    fn inner(&self) -> &Arc<IR::Context> {
        &self.inner
    }
}
