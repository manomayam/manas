//! I define an implementation of [`RepoContext`] for [`DerivedContentNegotiatingRepo`](crate::dconneging::DerivedContentNegotiatingRepo).
//!

use std::{fmt::Debug, sync::Arc};

use manas_repo::{
    context::{LayeredRepoContext, RepoContext},
    Repo, RepoResourceReader,
};

use super::{conneg_layer::DerivedContentNegotiationLayer, MRepo};

/// An implementation of [`RepoContext`] for [`DerivedContentNegotiatingRepo`](crate::dconneging::DerivedContentNegotiatingRepo).
pub struct DerivedContentNegotiatingRepoContext<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    /// Inner repo config.
    pub inner: Arc<IR::Context>,

    /// Derived conneg layer config.
    pub dconneg_layer_config: Arc<CNL::Config>,
}

impl<IR, CNL> Debug for DerivedContentNegotiatingRepoContext<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedContentNegotiatingRepoContext")
            .field("inner", &self.inner)
            .field("dconneg_reader_config", &self.dconneg_layer_config)
            .finish()
    }
}

impl<IR, CNL> RepoContext for DerivedContentNegotiatingRepoContext<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    type Repo = MRepo<IR, CNL>;

    #[inline]
    fn storage_space(&self) -> &Arc<IR::StSpace> {
        self.inner.storage_space()
    }
}

impl<IR, CNL> LayeredRepoContext for DerivedContentNegotiatingRepoContext<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    type InnerRepo = IR;

    #[inline]
    fn inner(&self) -> &Arc<IR::Context> {
        &self.inner
    }
}
