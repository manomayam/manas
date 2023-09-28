//! I define an implementation of [`RepoContext`] for [`ValidatingRepo`](super::ValidatingRepo).
//!

use std::{fmt::Debug, sync::Arc};

use manas_repo::{
    context::{LayeredRepoContext, RepoContext},
    Repo,
};

use super::{update_validator::RepUpdateValidator, MRepo};

// use super::{service::resource_operator::reader::layer::DerivedContentNegotiationLayer, MRepo};

/// An implementation of [`RepoContext`] for [`ValidatingRepo`](super::ValidatingRepo).
#[derive(Debug)]
pub struct ValidatingRepoContext<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    /// Inner repo config.
    pub inner: Arc<IR::Context>,

    /// Rep update validator config.
    pub rep_update_validator_config: Arc<V::Config>,
}

impl<IR, V> RepoContext for ValidatingRepoContext<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type Repo = MRepo<IR, V>;

    #[inline]
    fn storage_space(&self) -> &Arc<IR::StSpace> {
        self.inner.storage_space()
    }
}

impl<IR, V> LayeredRepoContext for ValidatingRepoContext<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type InnerRepo = IR;

    #[inline]
    fn inner(&self) -> &Arc<IR::Context> {
        &self.inner
    }
}
