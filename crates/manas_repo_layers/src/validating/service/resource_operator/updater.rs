//! I provide an implementation of [`ResourceUpdater`] for [`ValidatingRepo`].
//!

use std::{marker::PhantomData, task::Poll};

use dyn_problem::{type_::INTERNAL_ERROR, ProbFuture, Problem};
use futures::TryFutureExt;
use manas_repo::{
    context::LayeredRepoContext,
    service::resource_operator::{
        common::{
            problem::UNSUPPORTED_OPERATION, rep_update_action::RepUpdateAction,
            status_token::impl_::layered::Layered,
        },
        updater::{
            ResourceUpdateRequest, ResourceUpdateResponse, ResourceUpdateTokenSet, ResourceUpdater,
        },
    },
    Repo, RepoResourceUpdater,
};
use tower::{Service, ServiceExt};
use tracing::error;

use crate::validating::{
    update_validator::{update_context::RepUpdateContext, RepUpdateValidator},
    ValidatingRepo,
};

/// An implementation of [`ResourceUpdater`] for [`ValidatingRepo`]
#[derive(Debug)]
pub struct ValidatingRepoResourceUpdater<IR: Repo, V> {
    inner: RepoResourceUpdater<IR>,
    _phantom: PhantomData<fn(V)>,
}

impl<IR: Repo, V> Default for ValidatingRepoResourceUpdater<IR, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<IR: Repo, V> Clone for ValidatingRepoResourceUpdater<IR, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<IR, V> Service<ResourceUpdateRequest<ValidatingRepo<IR, V>>>
    for ValidatingRepoResourceUpdater<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type Response = ResourceUpdateResponse;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(skip_all, name = "ValidatingRepoResourceUpdater::call")]
    fn call(&mut self, req: ResourceUpdateRequest<ValidatingRepo<IR, V>>) -> Self::Future {
        let mut inner_svc = self.inner.clone();

        Box::pin(async move {
            let effective_new_rep = match req.rep_update_action {
                RepUpdateAction::SetWith(rep) => rep,
                RepUpdateAction::PatchWith(_) => {
                    error!("Validating repo doesn't support patch operation.");
                    return Err(UNSUPPORTED_OPERATION.new_problem());
                }
            };

            let Layered {
                inner: inner_req_tokens,
                layer_context,
            } = Layered::from(req.tokens);

            let mut update_context = RepUpdateContext::<IR> {
                res_slot: inner_req_tokens.res_token.slot().clone(),
                repo_context: layer_context.inner().clone(),
                current_res_token: Some(inner_req_tokens.res_token),
                effective_new_rep,
                op_req_extensions: req.extensions,
            };

            // Validate rep update.
            update_context = V::new(layer_context.as_ref().rep_update_validator_config.clone())
                .ready()
                .and_then(|svc| svc.call(update_context))
                .inspect_err(|e| error!("Rep update validation error. {e}"))
                .await?;

            let inner_req = ResourceUpdateRequest::<IR> {
                tokens: ResourceUpdateTokenSet {
                    res_token: update_context.current_res_token.ok_or_else(|| {
                        error!("Validators swapped resource token. Not allowed.");
                        INTERNAL_ERROR.new_problem()
                    })?,
                },
                rep_update_action: RepUpdateAction::SetWith(update_context.effective_new_rep),
                preconditions: req.preconditions,
                credentials: req.credentials,
                extensions: update_context.op_req_extensions,
            };

            inner_svc.ready().await?.call(inner_req).await
        })
    }
}

impl<IR, V> ResourceUpdater for ValidatingRepoResourceUpdater<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type Repo = ValidatingRepo<IR, V>;
}
