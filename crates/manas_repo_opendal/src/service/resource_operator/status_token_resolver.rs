//! I provide an implementation of [`ResourceStatusTokenResolver`] for ODR.
//!

use std::{sync::Arc, task::Poll};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, Problem};
use futures::TryFutureExt;
use manas_repo::{
    context::RepoContextual,
    service::resource_operator::status_token_resolver::{
        ResourceStatusTokenRequest, ResourceStatusTokenResolver, ResourceStatusTokenResponse,
    },
};
use tower::Service;
use tracing::error;

use crate::{
    context::ODRContext,
    service::resource_operator::common::status_token::ODRBaseResourceStatusToken, setup::ODRSetup,
    OpendalRepo,
};

/// An implementation of [`ResourceStatusTokenResolver`] for ODR.
#[derive(Debug, Clone)]
pub struct ODRResourceStatusTokenResolver<Setup: ODRSetup> {
    /// Repo context.
    repo_context: Arc<ODRContext<Setup>>,
}

impl<Setup: ODRSetup> RepoContextual for ODRResourceStatusTokenResolver<Setup> {
    type Repo = OpendalRepo<Setup>;

    #[inline]
    fn new_with_context(repo_context: Arc<ODRContext<Setup>>) -> Self {
        Self { repo_context }
    }

    #[inline]
    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }
}

impl<Setup: ODRSetup> Service<ResourceStatusTokenRequest>
    for ODRResourceStatusTokenResolver<Setup>
{
    type Response = ResourceStatusTokenResponse<OpendalRepo<Setup>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(
        skip_all,
        name = "ODRResourceStatusTokenResolver<Setup>::call",
        fields(req)
    )]
    fn call(&mut self, req: ResourceStatusTokenRequest) -> Self::Future {
        let repo_context = self.repo_context.clone();

        Box::pin(async move {
            // Resolve base token.
            let base_token: ODRBaseResourceStatusToken<Setup> =
                ODRBaseResourceStatusToken::<Setup>::try_current_for(
                    repo_context.clone(),
                    req.resource_uri,
                )
                .map_err(|e| {
                    error!("Unknown io error in resolving base token. Error:\n {}", e);
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })
                .await?;

            // Convert into token.
            let token = async_convert::TryFrom::try_from(base_token)
                .map_err(|e| {
                    error!(
                        "Unknown io error in resolving token from base token. Error:\n {}",
                        e
                    );
                    UNKNOWN_IO_ERROR.new_problem_builder().source(e).finish()
                })
                .await?;

            Ok(ResourceStatusTokenResponse { token })
        })
    }
}

impl<Setup: ODRSetup> ResourceStatusTokenResolver for ODRResourceStatusTokenResolver<Setup> {}
