//! I provide an implementation of [`RepPatcherResolver`]
//! for patching non supported repos.
//!

use std::{collections::HashSet, sync::Arc, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_space::resource::operation::SolidResourceOperation;
use tower::Service;
use tracing::{error, warn};

use crate::{
    context::RepoContextual,
    service::{
        patcher_resolver::RepPatcherResolver,
        resource_operator::common::{problem::UNSUPPORTED_OPERATION, rep_patcher::RepPatcher},
    },
    Repo,
};

/// An implementation of [`RepPatcher`] for patching non supported repos.
#[derive(Debug, Clone)]
pub struct UnsupportedRepPatcher(());

impl RepPatcher for UnsupportedRepPatcher {
    fn effective_ops(&self) -> HashSet<SolidResourceOperation> {
        unreachable!("Struct is not instantiable.")
    }
}

/// an implementation of [`RepPatcherResolver`] that
/// delegates to an inner repo rep patcher resolver.
#[derive(Debug)]
pub struct UnsupportedRepPatcherResolver<R: Repo> {
    repo_context: Arc<R::Context>,
}

impl<R: Repo> Clone for UnsupportedRepPatcherResolver<R> {
    fn clone(&self) -> Self {
        Self {
            repo_context: self.repo_context.clone(),
        }
    }
}

impl<R> RepoContextual for UnsupportedRepPatcherResolver<R>
where
    R: Repo<RepPatcher = UnsupportedRepPatcher>,
{
    type Repo = R;

    #[inline]
    fn new_with_context(context: Arc<<Self::Repo as Repo>::Context>) -> Self {
        warn!("Unsupported rep patcher resolver has been instantiated.");
        Self {
            repo_context: context,
        }
    }

    #[inline]
    fn repo_context(&self) -> &Arc<<Self::Repo as Repo>::Context> {
        &self.repo_context
    }
}

impl<R> Service<BinaryRepresentation> for UnsupportedRepPatcherResolver<R>
where
    R: Repo<RepPatcher = UnsupportedRepPatcher>,
{
    type Response = R::RepPatcher;

    type Error = Problem;

    type Future = ProbFuture<'static, R::RepPatcher>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, _rep: BinaryRepresentation) -> Self::Future {
        Box::pin(async move {
            error!("Unsupported patcher resolver had been called to resolve.");
            Err(UNSUPPORTED_OPERATION
                .new_problem_builder()
                .message("Patch operation is not supported.")
                .finish())
        })
    }
}

impl<R> RepPatcherResolver for UnsupportedRepPatcherResolver<R> where
    R: Repo<RepPatcher = UnsupportedRepPatcher>
{
}
