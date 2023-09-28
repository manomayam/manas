//! I provide an implementation of [`ResourceUpdater`] for [`PatchingRepo`].
//!

use std::{marker::PhantomData, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use manas_repo::{
    service::{
        patcher_resolver::impl_::UnsupportedRepPatcher,
        resource_operator::{
            common::rep_update_action::RepUpdateAction,
            updater::{
                ResourceUpdateRequest, ResourceUpdateResponse, ResourceUpdateTokenSet,
                ResourceUpdater,
            },
        },
    },
    Repo, RepoResourceUpdater,
};
use tower::{Service, ServiceExt};
use tracing::error;

use crate::patching::{
    patcher::{util::try_deref_resolve_effective_rep, DirectRepPatcher},
    PatchingRepo,
};

/// An implementation of [`ResourceUpdater`] for [`PatchingRepo`]
#[derive(Debug)]
pub struct PatchingRepoResourceUpdater<IR: Repo, P> {
    inner: RepoResourceUpdater<IR>,
    _phantom: PhantomData<fn(P)>,
}

impl<IR: Repo, P> Default for PatchingRepoResourceUpdater<IR, P> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<IR: Repo, P> Clone for PatchingRepoResourceUpdater<IR, P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<IR, P> Service<ResourceUpdateRequest<PatchingRepo<IR, P>>>
    for PatchingRepoResourceUpdater<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Response = ResourceUpdateResponse;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(skip_all, name = "PatchingRepoResourceUpdater::call")]
    fn call(&mut self, mut req: ResourceUpdateRequest<PatchingRepo<IR, P>>) -> Self::Future {
        let mut inner_svc = self.inner.clone();

        Box::pin(async move {
            let (res_token, effective_rep) =
                try_deref_resolve_effective_rep(req.tokens.res_token, req.rep_update_action)
                    .inspect_err(|e| {
                        error!("Error in resolving patch effective rep. {e}");
                    })
                    .await?;

            req.tokens = ResourceUpdateTokenSet { res_token };
            req.rep_update_action = RepUpdateAction::SetWith(effective_rep);

            let inner_req = req.unlayer_tokens();
            inner_svc.ready().await?.call(inner_req).await
        })
    }
}

impl<IR, P> ResourceUpdater for PatchingRepoResourceUpdater<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Repo = PatchingRepo<IR, P>;
}
