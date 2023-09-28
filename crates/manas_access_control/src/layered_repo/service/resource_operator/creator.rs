//! I provide an implementation of [`ResourceCreator`] for ACR.
//!

use std::{fmt::Debug, marker::PhantomData};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt};
use futures::TryFutureExt;
use manas_repo::{
    service::resource_operator::{
        common::{problem::ACCESS_DENIED, status_token::ExistingRepresentedResourceToken},
        creator::{ResourceCreateRequest, ResourceCreateResponse, ResourceCreator},
    },
    Repo, RepoResourceCreator,
};
use manas_space::resource::operation::SolidResourceOperation;
use tower::{Service, ServiceExt};
use tracing::{debug, error};
use typed_record::TypedRecord;

use crate::{
    layered_repo::AccessControlledRepo,
    model::{
        pep::PolicyEnforcementPoint, ActionOpList, JustifiedOperation, KResolvedHostAccessControl,
        ResolvedAccessControl,
    },
};

/// An implementation of [`ResourceCreator`] that checks for
/// access control for resource creation before
/// forwarding to inner.
pub struct AccessControlledResourceCreator<IR: Repo, PEP> {
    /// Inner resource creator.
    inner: RepoResourceCreator<IR>,
    _phantom: PhantomData<PEP>,
}

impl<IR: Repo, PEP> Default for AccessControlledResourceCreator<IR, PEP> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Clone for AccessControlledResourceCreator<IR, PEP>
where
    IR: Repo,
    RepoResourceCreator<IR>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Debug for AccessControlledResourceCreator<IR, PEP>
where
    IR: Repo,
    RepoResourceCreator<IR>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessControlledResourceCreator")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<IR, PEP> Service<ResourceCreateRequest<AccessControlledRepo<IR, PEP>>>
    for AccessControlledResourceCreator<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Response = ResourceCreateResponse<AccessControlledRepo<IR, PEP>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "AccessControlledResourceCreator::call")]
    fn call(&mut self, req: ResourceCreateRequest<AccessControlledRepo<IR, PEP>>) -> Self::Future {
        let layer_context = req.tokens.repo_context().clone();

        let host_res_uri = req.tokens.host_token().slot().id().uri.clone();

        // Get credentials.
        let credentials = req.credentials.clone();

        // Translate request for inner service.
        let inner_req = req.unlayer_tokens();

        // Get inner service
        let mut inner_svc = self.inner.clone();

        Box::pin(async move {
            // Check with enforcement point for access.
            let mut action_host_ops = vec![JustifiedOperation {
                op: SolidResourceOperation::APPEND,
                why: "To append a child resource.".into(),
            }];

            if !inner_req.host_preconditions.are_trivial() {
                action_host_ops.push(JustifiedOperation {
                    op: SolidResourceOperation::READ,
                    why: "To evaluate non-trivial preconditions.".into(),
                });
            }

            let resolved_host_access_control: ResolvedAccessControl<_> = layer_context
                .as_ref()
                .pep
                .resolve_access_control(
                    ActionOpList {
                        on: host_res_uri,
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

            debug!(
                "Resolved host access control: {:?}",
                resolved_host_access_control
            );

            if !resolved_host_access_control.is_allowed() {
                error!("Access denied for create operation.");
                return Err(ACCESS_DENIED
                    .new_problem_builder()
                    .extend_with::<KResolvedHostAccessControl<_>>(resolved_host_access_control)
                    .finish());
            }

            // Call inner service.
            let mut resp: ResourceCreateResponse<_> = inner_svc
                .ready()
                .and_then(|svc| svc.call(inner_req))
                .await
                .map_err(|mut e: Problem| {
                    e.extensions_mut()
                        .insert_rec_item::<KResolvedHostAccessControl<_>>(
                            resolved_host_access_control.clone(),
                        );
                    e
                })?;

            // Attach resolved access token to extensions.
            resp.extensions
                .insert_rec_item::<KResolvedHostAccessControl<_>>(resolved_host_access_control);

            Ok(resp.map_repo())
        })
    }
}

impl<IR, PEP> ResourceCreator for AccessControlledResourceCreator<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Repo = AccessControlledRepo<IR, PEP>;
}
