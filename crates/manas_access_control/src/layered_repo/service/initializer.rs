//! I provide an implementation of [`RepoInitializer`] for
//! access controlled repo.
//!

use std::sync::Arc;

use dyn_problem::{type_::INTERNAL_ERROR, ProbFuture, Problem};
use futures::TryFutureExt;
use manas_repo::{
    context::{RepoContext, RepoContextual},
    service::{
        initializer::RepoInitializer,
        resource_operator::{
            common::{
                rep_update_action::RepUpdateAction,
                status_token::{ExistingResourceToken, ResourceStatusToken},
            },
            reader::{rep_preferences::RepresentationPreferences, ResourceReadResponse},
            updater::{ResourceUpdateRequest, ResourceUpdateTokenSet},
        },
    },
    Repo, RepoExt, RepoInitializerService, RepoResourceStatusToken, RepoResourceUpdater,
};
use manas_space::{resource::uri::SolidResourceUri, SolidStorageSpace};
use tower::{Service, ServiceExt};
use tracing::{debug, error, info, warn};

use crate::{
    layered_repo::{context::AccessControlledRepoContext, AccessControlledRepo},
    model::pep::PolicyEnforcementPoint,
};

/// An implementation of [`RepoInitializer`] for
/// access controlled repo.
#[derive(Debug)]
pub struct AccessControlledRepoInitializer<IR: Repo, PEP> {
    inner: RepoInitializerService<IR>,
    layer_context: Arc<AccessControlledRepoContext<IR, PEP>>,
}

impl<IR, PEP> RepoContextual for AccessControlledRepoInitializer<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Repo = AccessControlledRepo<IR, PEP>;

    #[inline]
    fn new_with_context(context: Arc<AccessControlledRepoContext<IR, PEP>>) -> Self {
        Self {
            inner: RepoContextual::new_with_context(context.inner.clone()),
            layer_context: context,
        }
    }

    #[inline]
    fn repo_context(&self) -> &Arc<AccessControlledRepoContext<IR, PEP>> {
        &self.layer_context
    }
}

impl<IR, PEP> Service<()> for AccessControlledRepoInitializer<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Response = bool;

    type Error = Problem;

    type Future = ProbFuture<'static, bool>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "AccessControlledRepoInitializer::call")]
    fn call(&mut self, req: ()) -> Self::Future {
        let layer_context = self.layer_context.clone();
        let inner_fut = self.inner.call(req);

        Box::pin(async move {
            // Call inner initializer first.
            let resp: bool = inner_fut.await?;
            let layer_context = layer_context.as_ref();

            let inner_repo = IR::new(layer_context.inner.clone());

            // Read storage root state.
            let stroot_read_resp: ResourceReadResponse<IR, IR::Representation> = inner_repo
                .read_basic(
                    layer_context.storage_space().root_res_uri().clone(),
                    Default::default(),
                    RepresentationPreferences::new_light(),
                )
                .inspect_err(|e| error!("Error in reading storage root state. Error:\n {}", e))
                .await?
                .ok_or_else(|| {
                    error!("Storage root is not initialized by inner repo initializer.");
                    INTERNAL_ERROR
                        .new_problem_builder()
                        .message("Storage root is not initialized by inner repo initializer.")
                        .finish()
                })?;

            // Resolve root acr uri.
            let stroot_acr_uri: SolidResourceUri = if let Some(link) = stroot_read_resp
                .aux_links_index
                .iter()
                .find(|l| l.is_acl_link())
            {
                link.target.clone()
            } else {
                warn!("No acr link for storage root.");
                return Ok(resp);
            };

            let initial_root_acr_rep: IR::Representation = if let Some(rep) =
                (layer_context.initial_root_acr_rep_factory)(stroot_acr_uri.clone())
            {
                rep
            } else {
                debug!("Initial root acr not configured");
                return Ok(resp);
            };

            // Get status token for root acr.
            let stroot_acr_token: RepoResourceStatusToken<IR> = inner_repo
                .resolve_status_token(stroot_acr_uri)
                .inspect_err(|e| error!("Error in resolving root acr status token. Error:\n {}", e))
                .await?;

            let e_stroot_acr_token = if let ResourceStatusToken::Existing(t) = stroot_acr_token {
                if matches!(t, ExistingResourceToken::Represented(_)) {
                    info!("Root acr is already represented.");
                    return Ok(resp);
                }
                t
            } else {
                error!("Acr slot was not created for storage root acr");
                return Err(INTERNAL_ERROR
                    .new_problem_builder()
                    .message("Acr slot was not created for storage root acr")
                    .finish());
            };

            // Set initial representation.
            RepoResourceUpdater::<IR>::default()
                .ready()
                .and_then(|svc| {
                    svc.call(ResourceUpdateRequest {
                        tokens: ResourceUpdateTokenSet::new(e_stroot_acr_token),
                        rep_update_action: RepUpdateAction::SetWith(initial_root_acr_rep),
                        credentials: Default::default(),
                        preconditions: Box::new(()),
                        extensions: Default::default(),
                    })
                })
                .inspect_err(|e| {
                    error!("Error in updating root acr representation. Error:\n {}", e)
                })
                .await?;

            Ok(true)
        })
    }
}

impl<IR, PEP> RepoInitializer for AccessControlledRepoInitializer<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
}
