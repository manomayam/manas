//! I provide an implementation of [`RepoInitializer`] that
//! delegates to an inner repo initializer.
//!

use std::task::Poll;

use dyn_problem::{ProbFuture, Problem};
use tower::Service;

use crate::{
    context::{impl_::DelegatedRepoContextual, LayeredRepoContext},
    service::initializer::RepoInitializer,
    Repo,
};

/// an implementation of [`RepoInitializer`] that
/// delegates to an inner repo initializer.
pub type DelegatedRepoInitializer<Inner, LR> = DelegatedRepoContextual<Inner, LR>;

impl<Inner, LR> Service<()> for DelegatedRepoInitializer<Inner, LR>
where
    Inner: RepoInitializer,
    LR: Repo,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
    type Response = bool;

    type Error = Problem;

    type Future = ProbFuture<'static, bool>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<()>::poll_ready(&mut self.inner, cx)
    }

    #[inline]
    fn call(&mut self, req: ()) -> Self::Future {
        Service::<()>::call(&mut self.inner, req)
    }
}

impl<Inner, LR> RepoInitializer for DelegatedRepoContextual<Inner, LR>
where
    Inner: RepoInitializer,
    LR: Repo,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
}
