//! I provide an implementation of [`RepoContext`] that  
//! delegates to inner context.
//!

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use crate::{
    context::{LayeredRepoContext, RepoContext, RepoContextual},
    Repo,
};

/// A type to represent delegated repo context.
pub struct DelegatedRepoContext<IR: Repo, LR> {
    inner: Arc<IR::Context>,
    _phantom: PhantomData<fn(LR)>,
}

impl<IR, LR> Debug for DelegatedRepoContext<IR, LR>
where
    IR: Repo,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatedRepoContext")
            .field("inner", &self.inner)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<IR: Repo, LR> Clone for DelegatedRepoContext<IR, LR> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<R, LR> RepoContext for DelegatedRepoContext<R, LR>
where
    R: Repo,
    LR: Repo<StSpace = R::StSpace>,
{
    type Repo = LR;

    #[inline]
    fn storage_space(&self) -> &Arc<<LR as Repo>::StSpace> {
        self.inner.storage_space()
    }
}

impl<R: Repo, LR> DelegatedRepoContext<R, LR> {
    /// Create a new instance of [`DelegatedRepoContext`].
    #[inline]
    pub fn new(inner: Arc<R::Context>) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<R, LR> LayeredRepoContext for DelegatedRepoContext<R, LR>
where
    R: Repo,
    LR: Repo<StSpace = R::StSpace, Context = Self>,
{
    type InnerRepo = R;

    #[inline]
    fn inner(&self) -> &Arc<R::Context> {
        &self.inner
    }
}

/// A type to represent delegated repo contextual.
pub struct DelegatedRepoContextual<Inner, LR: Repo> {
    pub(crate) inner: Inner,
    pub(crate) repo_context: Arc<LR::Context>,
    _phantom: PhantomData<fn(LR)>,
}

impl<Inner: Debug, LR: Repo> Debug for DelegatedRepoContextual<Inner, LR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatedRepoContextual")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<Inner: Clone, LR: Repo> Clone for DelegatedRepoContextual<Inner, LR> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            repo_context: self.repo_context.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<Inner, LR> RepoContextual for DelegatedRepoContextual<Inner, LR>
where
    Inner: RepoContextual,
    LR: Repo,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
    type Repo = LR;

    #[inline]
    fn new_with_context(context: Arc<<LR as Repo>::Context>) -> Self {
        Self::new(
            RepoContextual::new_with_context(context.inner().clone()),
            context,
        )
    }

    #[inline]
    fn repo_context(&self) -> &Arc<<LR as Repo>::Context> {
        &self.repo_context
    }
}

impl<Inner, LR: Repo> DelegatedRepoContextual<Inner, LR> {
    /// Create a new [`DelegatedRepoContextual`].
    #[inline]
    pub fn new(inner: Inner, repo_context: Arc<LR::Context>) -> Self {
        Self {
            inner,
            repo_context,
            _phantom: PhantomData,
        }
    }
}
