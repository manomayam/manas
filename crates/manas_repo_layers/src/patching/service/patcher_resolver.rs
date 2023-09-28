//! I provide an implementation of [`RepPatcherResolver`] for [`PatchingRepo`].
//!

use std::{sync::Arc, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_repo::{
    context::RepoContextual,
    service::patcher_resolver::{impl_::UnsupportedRepPatcher, RepPatcherResolver},
    Repo,
};
use tower::Service;

use crate::patching::{context::PatchingRepoContext, patcher::DirectRepPatcher, PatchingRepo};

/// An implementation of [`RepPatcherResolver`] for [`PatchingRepo`].
#[derive(Debug)]
pub struct DirectRepPatcherResolver<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    /// Repo context.
    repo_context: Arc<PatchingRepoContext<IR, P>>,
}

impl<IR, P> Clone for DirectRepPatcherResolver<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    fn clone(&self) -> Self {
        Self {
            repo_context: self.repo_context.clone(),
        }
    }
}

impl<IR, P> RepoContextual for DirectRepPatcherResolver<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Repo = PatchingRepo<IR, P>;

    #[inline]
    fn new_with_context(context: Arc<<Self::Repo as Repo>::Context>) -> Self {
        Self {
            repo_context: context,
        }
    }

    #[inline]
    fn repo_context(&self) -> &Arc<<Self::Repo as Repo>::Context> {
        &self.repo_context
    }
}

impl<IR, P> Service<BinaryRepresentation> for DirectRepPatcherResolver<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Response = P;

    type Error = Problem;

    type Future = ProbFuture<'static, P>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, rep: BinaryRepresentation) -> Self::Future {
        P::try_resolve(
            rep,
            self.repo_context.as_ref().patcher_resolution_config.clone(),
        )
    }
}

impl<IR, P> RepPatcherResolver for DirectRepPatcherResolver<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
}
