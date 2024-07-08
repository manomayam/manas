//! I define traits for repo layers.
//!

use std::{fmt::Debug, sync::Arc};

use crate::{
    context::LayeredRepoContext,
    service::resource_operator::common::status_token::impl_::layered::LayeredResourceStatusTokenTypes,
    Repo,
};

/// A trait for layered repos.
pub trait LayeredRepo<Inner: Repo>:
    Repo<
    StSpace = Inner::StSpace,
    ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
        Inner::ResourceStatusTokenTypes,
        Self,
    >,
    Context: LayeredRepoContext<InnerRepo = Inner, Repo = Self>,
>
{
}

/// A trait for the repo layers.
pub trait RepoLayer<InnerRepo: Repo>: Debug + Send + Sync + 'static {
    /// Type of the layered repo.
    type LayeredRepo: LayeredRepo<InnerRepo>;

    /// Layer the inner repo context.
    fn layer_context(
        &self,
        inner_context: Arc<InnerRepo::Context>,
    ) -> <Self::LayeredRepo as Repo>::Context;

    /// Layer the inner repo.
    fn layer(&self, inner_repo: InnerRepo) -> Self::LayeredRepo {
        <Self::LayeredRepo as Repo>::new(Arc::new(self.layer_context(inner_repo.context().clone())))
    }
}

impl<IR, LR> LayeredRepo<IR> for LR
where
    IR: Repo,
    LR: Repo<
        StSpace = IR::StSpace,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            IR::ResourceStatusTokenTypes,
            Self,
        >,
        Context: LayeredRepoContext<InnerRepo = IR, Repo = LR>,
    >,
{
}
