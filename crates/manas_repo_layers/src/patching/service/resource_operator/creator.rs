//! I provide an implementation of [`ResourceCreator`] for [`PatchingRepo`].
//!

use std::{marker::PhantomData, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use manas_repo::{
    service::{
        patcher_resolver::impl_::UnsupportedRepPatcher,
        resource_operator::{
            common::rep_update_action::RepUpdateAction,
            creator::{
                ResourceCreateRequest, ResourceCreateResponse, ResourceCreator,
                SLOT_REL_SUBJECT_CONSTRAIN_VIOLATION,
            },
        },
    },
    Repo, RepoResourceCreator,
};
use manas_space::resource::state::SolidResourceState;
use tower::{Service, ServiceExt};
use tracing::error;

use crate::patching::{
    patcher::{util::try_resolve_effective_rep, DirectRepPatcher},
    PatchingRepo,
};

/// An implementation of [`ResourceCreator`] for [`PatchingRepo`]
#[derive(Debug)]
pub struct PatchingRepoResourceCreator<IR: Repo, P> {
    inner: RepoResourceCreator<IR>,
    _phantom: PhantomData<fn(P)>,
}

impl<IR: Repo, P> Default for PatchingRepoResourceCreator<IR, P> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<IR: Repo, P> Clone for PatchingRepoResourceCreator<IR, P> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<IR, P> Service<ResourceCreateRequest<PatchingRepo<IR, P>>>
    for PatchingRepoResourceCreator<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Response = ResourceCreateResponse<PatchingRepo<IR, P>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(skip_all, name = "PatchingRepoResourceCreator::call")]
    fn call(&mut self, mut req: ResourceCreateRequest<PatchingRepo<IR, P>>) -> Self::Future {
        let mut inner_svc = self.inner.clone();
        Box::pin(async move {
            let new_res_slot = req.try_equivalent_res_slot().map_err(|e| {
                error!("Invalid new resource slot. {e}");
                SLOT_REL_SUBJECT_CONSTRAIN_VIOLATION.new_problem()
            })?;

            // Modify the update action to `set_with` variant by resolving patch.
            req.rep_update_action = RepUpdateAction::SetWith(
                try_resolve_effective_rep(
                    SolidResourceState {
                        slot: new_res_slot,
                        representation: None,
                    },
                    req.rep_update_action,
                )
                .inspect_err(|e| error!("Error in resolving patch effective rep. {e}"))
                .await?,
            );

            // Then pass on to inner service.
            let inner_req = req.unlayer_tokens();
            Ok(inner_svc
                .ready()
                .and_then(|svc| svc.call(inner_req))
                .await?
                .map_repo())
        })
    }
}

impl<IR, P> ResourceCreator for PatchingRepoResourceCreator<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Repo = PatchingRepo<IR, P>;
}
