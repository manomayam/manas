//! I provide an implementation of [`ResourceReader`] for ACR.
//!

use std::{fmt::Debug, marker::PhantomData};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt};
use futures::TryFutureExt;
use manas_repo::{
    service::resource_operator::{
        common::{problem::ACCESS_DENIED, status_token::ExistingRepresentedResourceToken},
        reader::{
            FlexibleResourceReader, ResourceReadRequest, ResourceReadResponse, ResourceReader,
        },
    },
    Repo, RepoResourceReader,
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

/// An implementation of [`ResourceReader`] that checks for
/// access control for read operation over resource before
/// forwarding to inner.
pub struct AccessControlledResourceReader<IR: Repo, PEP> {
    /// Inner resource reader.
    inner: RepoResourceReader<IR>,
    _phantom: PhantomData<PEP>,
}

impl<IR: Repo, PEP> Default for AccessControlledResourceReader<IR, PEP> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Clone for AccessControlledResourceReader<IR, PEP>
where
    IR: Repo,
    RepoResourceReader<IR>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> Debug for AccessControlledResourceReader<IR, PEP>
where
    IR: Repo,
    RepoResourceReader<IR>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessControlledResourceReader")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<IR, PEP> Service<ResourceReadRequest<AccessControlledRepo<IR, PEP>>>
    for AccessControlledResourceReader<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Response = ResourceReadResponse<AccessControlledRepo<IR, PEP>, IR::Representation>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "AccessControlledResourceReader::call")]
    fn call(&mut self, req: ResourceReadRequest<AccessControlledRepo<IR, PEP>>) -> Self::Future {
        let layer_context = req.tokens.res_token.layer_context.clone();

        let credentials = req.credentials.clone();
        let res_uri = req.tokens.res_token.slot().id().uri.clone();

        // Translate request for inner service.
        let inner_req = req.unlayer_tokens();

        // Get inner service
        let mut inner_svc = self.inner.clone();

        Box::pin(async move {
            // Check with enforcement point for access.
            let action_op_list = ActionOpList {
                on: res_uri,
                ops: vec![JustifiedOperation {
                    op: SolidResourceOperation::READ,
                    why: "To read the resource state.".into(),
                }],
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
                error!("Access denied for read operation.");
                return Err(ACCESS_DENIED
                    .new_problem_builder()
                    .extend_with::<KResolvedAccessControl<_>>(resolved_access_control)
                    .finish());
            }

            // Call inner service.
            let mut resp: ResourceReadResponse<_, _> = inner_svc
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

            Ok(resp.layer_tokens(layer_context))
        })
    }
}

impl<IR, PEP> FlexibleResourceReader<AccessControlledRepo<IR, PEP>, IR::Representation>
    for AccessControlledResourceReader<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
}

impl<IR, PEP> ResourceReader for AccessControlledResourceReader<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Repo = AccessControlledRepo<IR, PEP>;
}
