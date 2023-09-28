//! I provide an implementation of [`ResourceCreator`] for [`ValidatingRepo`].
//!

use std::{marker::PhantomData, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use manas_repo::{
    context::LayeredRepoContext,
    service::resource_operator::{
        common::{
            problem::UNSUPPORTED_OPERATION, rep_update_action::RepUpdateAction,
            status_token::impl_::layered::Layered,
        },
        creator::{
            ResourceCreateRequest, ResourceCreateResponse, ResourceCreator,
            SLOT_REL_SUBJECT_CONSTRAIN_VIOLATION,
        },
    },
    Repo, RepoResourceCreator,
};
use tower::{Service, ServiceExt};
use tracing::error;

use crate::validating::{
    update_validator::{update_context::RepUpdateContext, RepUpdateValidator},
    ValidatingRepo,
};

/// An implementation of [`ResourceCreator`] for [`ValidatingRepo`]
#[derive(Debug)]
pub struct ValidatingRepoResourceCreator<IR: Repo, V> {
    inner: RepoResourceCreator<IR>,
    _phantom: PhantomData<fn(V)>,
}

impl<IR: Repo, V> Default for ValidatingRepoResourceCreator<IR, V> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<IR: Repo, V> Clone for ValidatingRepoResourceCreator<IR, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<IR, V> Service<ResourceCreateRequest<ValidatingRepo<IR, V>>>
    for ValidatingRepoResourceCreator<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type Response = ResourceCreateResponse<ValidatingRepo<IR, V>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(skip_all, name = "ValidatingRepoResourceCreator::call")]
    fn call(&mut self, req: ResourceCreateRequest<ValidatingRepo<IR, V>>) -> Self::Future {
        let mut inner_svc = self.inner.clone();
        Box::pin(async move {
            let new_res_slot = req.try_equivalent_res_slot().map_err(|e| {
                error!("Invalid new resource slot. {e}");
                SLOT_REL_SUBJECT_CONSTRAIN_VIOLATION.new_problem()
            })?;

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
                res_slot: new_res_slot,
                repo_context: layer_context.inner().clone(),
                current_res_token: None,
                effective_new_rep,
                op_req_extensions: req.extensions,
            };

            // Validate rep update.
            update_context = V::new(layer_context.as_ref().rep_update_validator_config.clone())
                .ready()
                .and_then(|svc| svc.call(update_context))
                .inspect_err(|e| error!("Rep update validation error. {e}"))
                .await?;

            let inner_req = ResourceCreateRequest {
                tokens: inner_req_tokens,
                resource_kind: req.resource_kind,
                slot_rev_rel_type: req.slot_rev_rel_type,
                rep_update_action: RepUpdateAction::SetWith(update_context.effective_new_rep),
                host_preconditions: req.host_preconditions,
                credentials: req.credentials,
                extensions: update_context.op_req_extensions,
            };

            Ok(inner_svc.ready().await?.call(inner_req).await?.map_repo())
        })
    }
}

impl<IR, V> ResourceCreator for ValidatingRepoResourceCreator<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type Repo = ValidatingRepo<IR, V>;
}
