//! I provide an implementation of [`ResourceUpdater`] for ACR.
//!

use std::{fmt::Debug, marker::PhantomData};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt};
use futures::TryFutureExt;
use manas_repo::{
    service::resource_operator::{
        common::{
            problem::ACCESS_DENIED, rep_patcher::RepPatcher, rep_update_action::RepUpdateAction,
        },
        updater::{ResourceUpdateRequest, ResourceUpdateResponse, ResourceUpdater},
    },
    Repo, RepoResourceUpdater,
};
use manas_space::resource::operation::SolidResourceOperation;
use tower::{Service, ServiceExt};
use tracing::{debug, error};
use typed_record::TypedRecord;

use crate::{
    layered_repo::AccessControlledRepo,
    model::{
        pep::PolicyEnforcementPoint, ActionOpList, JustifiedOperation, KResolvedAccessControl,
        ResolvedAccessControl,
    },
};

/// An implementation of [`ResourceUpdater`] that checks for
/// access control for resource update before
/// forwarding to inner.
pub struct AccessControlledResourceUpdater<IR: Repo, PEP> {
    /// Inner resource updater.
    inner: RepoResourceUpdater<IR>,
    _phantom: PhantomData<PEP>,
}

impl<IR: Repo, PEP> Default for AccessControlledResourceUpdater<IR, PEP> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Clone for AccessControlledResourceUpdater<IR, PEP>
where
    IR: Repo,
    RepoResourceUpdater<IR>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Debug for AccessControlledResourceUpdater<IR, PEP>
where
    IR: Repo,
    RepoResourceUpdater<IR>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessControlledResourceUpdater")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<IR, PEP> Service<ResourceUpdateRequest<AccessControlledRepo<IR, PEP>>>
    for AccessControlledResourceUpdater<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Response = ResourceUpdateResponse;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "AccessControlledResourceUpdater::call")]
    fn call(&mut self, req: ResourceUpdateRequest<AccessControlledRepo<IR, PEP>>) -> Self::Future {
        let layer_context = req.tokens.res_token.repo_context().clone();

        let credentials = req.credentials.clone();
        let res_uri = req.tokens.res_token.slot().id().uri.clone();

        // Translate request for inner service.
        let inner_req = req.unlayer_tokens();

        // Get inner service
        let mut inner_svc = self.inner.clone();

        Box::pin(async move {
            // resolve action ops.
            let mut action_ops = vec![];
            match &inner_req.rep_update_action {
                RepUpdateAction::SetWith { .. } => {
                    action_ops.push(JustifiedOperation {
                        op: SolidResourceOperation::WRITE,
                        why: "To overwrite resource representation.".into(),
                    });
                }
                RepUpdateAction::PatchWith(patcher) => {
                    action_ops.extend(patcher.effective_ops().into_iter().map(|op| {
                        JustifiedOperation {
                            op,
                            why: "To apply supplied patch to resource representation.".into(),
                        }
                    }))
                }
            };
            if !inner_req.preconditions.are_trivial() {
                action_ops.push(JustifiedOperation {
                    op: SolidResourceOperation::READ,
                    why: "To evaluate non-trivial preconditions.".into(),
                });
            }

            let action_op_list = ActionOpList {
                on: res_uri,
                ops: action_ops,
            };

            let resolved_access_control: ResolvedAccessControl<_> = layer_context
                .as_ref()
                .pep
                .resolve_access_control(action_op_list, credentials)
                .map_err(|e| {
                    error!(
                        "Unknown io error in resolving access control. Error:\n {}",
                        e
                    );
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })
                .await?
                .resolved;

            debug!("Resolved access control: {:?}", resolved_access_control);

            if !resolved_access_control.is_allowed() {
                error!("Access denied for update operation.");
                return Err(ACCESS_DENIED
                    .new_problem_builder()
                    .extend_with::<KResolvedAccessControl<_>>(resolved_access_control)
                    .finish());
            }

            // Call inner service.
            let mut resp: ResourceUpdateResponse = inner_svc
                .ready()
                .and_then(|svc| svc.call(inner_req))
                .await
                .map_err(|mut e: Problem| {
                    e.extensions_mut()
                        .insert_rec_item::<KResolvedAccessControl<_>>(
                            resolved_access_control.clone(),
                        );
                    e
                })?;

            // Attach resolved access token to extensions.
            resp.extensions
                .insert_rec_item::<KResolvedAccessControl<_>>(resolved_access_control);

            Ok(resp)
        })
    }
}

impl<IR, PEP> ResourceUpdater for AccessControlledResourceUpdater<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Repo = AccessControlledRepo<IR, PEP>;
}
