//! I provide an implementation of [`ResourceDeleter`] for ACR.
//!

use std::{fmt::Debug, marker::PhantomData};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt};
use futures::TryFutureExt;
use manas_repo::{
    service::resource_operator::{
        common::{problem::ACCESS_DENIED, status_token::ExistingRepresentedResourceToken},
        deleter::{ResourceDeleteRequest, ResourceDeleteResponse, ResourceDeleter},
    },
    Repo, RepoResourceDeleter,
};
use manas_space::resource::operation::SolidResourceOperation;
use tower::{Service, ServiceExt};
use tracing::{debug, error};
use typed_record::TypedRecord;

use crate::{
    layered_repo::AccessControlledRepo,
    model::{
        pep::PolicyEnforcementPoint, ActionOpList, JustifiedOperation, KResolvedAccessControl,
        KResolvedHostAccessControl, ResolvedAccessControl,
    },
};

/// An implementation of [`ResourceDeleter`] that checks for
/// access control for resource delete before
/// forwarding to inner.
pub struct AccessControlledResourceDeleter<IR: Repo, PEP> {
    /// Inner resource deleter.
    inner: RepoResourceDeleter<IR>,
    _phantom: PhantomData<PEP>,
}

impl<IR: Repo, PEP> Default for AccessControlledResourceDeleter<IR, PEP> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Clone for AccessControlledResourceDeleter<IR, PEP>
where
    IR: Repo,
    RepoResourceDeleter<IR>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Debug for AccessControlledResourceDeleter<IR, PEP>
where
    IR: Repo,
    RepoResourceDeleter<IR>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessControlledResourceDeleter")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<IR, PEP> Service<ResourceDeleteRequest<AccessControlledRepo<IR, PEP>>>
    for AccessControlledResourceDeleter<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Response = ResourceDeleteResponse<AccessControlledRepo<IR, PEP>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "AccessControlledResourceDeleter::call")]
    fn call(&mut self, req: ResourceDeleteRequest<AccessControlledRepo<IR, PEP>>) -> Self::Future {
        let layer_context = req.tokens.res_token.layer_context.clone();

        let credentials = req.credentials.clone();
        let res_uri = req.tokens.res_token.slot().id().uri.clone();
        let opt_slot_rev_link = req.tokens.res_token.slot().slot_rev_link().cloned();

        // Translate request for inner service.
        let inner_req = req.unlayer_tokens();

        // Get inner service
        let mut inner_svc = self.inner.clone();

        Box::pin(async move {
            // resolve action ops.
            let mut action_ops = vec![JustifiedOperation {
                op: SolidResourceOperation::DELETE,
                why: "To delete resource.".into(),
            }];

            if inner_req.tokens.res_token.slot().is_container_slot() {
                action_ops.push(JustifiedOperation {
                    op: SolidResourceOperation::READ,
                    why: "To check if container resource is empty or not.".into(),
                });
            }

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
                .resolve_access_control(action_op_list, credentials.clone())
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
                error!("Access denied for delete operation.");
                return Err(ACCESS_DENIED
                    .new_problem_builder()
                    .extend_with::<KResolvedAccessControl<_>>(resolved_access_control)
                    .finish());
            }

            if let Some(slot_rev_link) = opt_slot_rev_link {
                // If resource is a contained, then there
                // will be write op over container.
                if slot_rev_link.rev_rel_type.is_contains() {
                    let action_host_ops = vec![JustifiedOperation {
                        op: SolidResourceOperation::WRITE,
                        why: "To remove containment triple corresponding to child resource to be deleted.".into(),
                    }];

                    let resolved_host_access_control: ResolvedAccessControl<_> = layer_context
                        .as_ref()
                        .pep
                        .resolve_access_control(
                            ActionOpList {
                                on: slot_rev_link.target,
                                ops: action_host_ops,
                            },
                            credentials,
                        )
                        .map_err(|e| {
                            error!(
                                "Unknown io error in resolving access control. Error:\n {}",
                                e
                            );
                            UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                        })
                        .await?
                        .resolved;

                    if !resolved_host_access_control.is_allowed() {
                        error!("Access denied for delete operation, as write operation on container is not allowed.");
                        return Err(ACCESS_DENIED
                            .new_problem_builder()
                            .extend_with::<KResolvedHostAccessControl<_>>(
                                resolved_host_access_control,
                            )
                            .finish());
                    }
                }
            }

            // Call inner service.
            let mut resp: ResourceDeleteResponse<_> = inner_svc
                .ready()
                .and_then(|svc| svc.call(inner_req))
                .await
                .map_err(|mut e: Problem| {
                    e.extensions_mut()
                        .insert_rec_item::<KResolvedAccessControl<_>>(
                            resolved_access_control.clone(),
                        );
                    e
                })?
                .map_repo();

            // Attach resolved access token to extensions.
            resp.extensions
                .insert_rec_item::<KResolvedAccessControl<_>>(resolved_access_control);

            Ok(resp)
        })
    }
}

impl<IR, PEP> ResourceDeleter for AccessControlledResourceDeleter<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Repo = AccessControlledRepo<IR, PEP>;
}
