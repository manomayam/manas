//! I provide an implementation of
//! [`ResourceStatusTokenResolver`] that yields layered
//! status tokens over inner service yielded.
//!

use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use tower::Service;

use crate::{
    context::{impl_::DelegatedRepoContextual, LayeredRepoContext},
    service::resource_operator::{
        common::status_token::impl_::layered::{Layered, LayeredResourceStatusTokenTypes},
        status_token_resolver::{
            ResourceStatusTokenRequest, ResourceStatusTokenResolver, ResourceStatusTokenResponse,
        },
    },
    Repo,
};

/// An implementation of
/// [`ResourceStatusTokenResolver`] that yields layered
/// status tokens over inner service yielded.
pub type LayeredResourceStatusTokenResolver<Inner, LR> = DelegatedRepoContextual<Inner, LR>;

impl<Inner, LR> Service<ResourceStatusTokenRequest>
    for LayeredResourceStatusTokenResolver<Inner, LR>
where
    Inner: ResourceStatusTokenResolver,
    LR: Repo<
        StSpace = <Inner::Repo as Repo>::StSpace,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            <Inner::Repo as Repo>::ResourceStatusTokenTypes,
            LR,
        >,
    >,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
    type Response = ResourceStatusTokenResponse<LR>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ResourceStatusTokenRequest) -> Self::Future {
        let layer_context = self.repo_context.clone();
        Box::pin(
            self.inner
                .call(req)
                .map_ok(|resp| ResourceStatusTokenResponse {
                    token: Layered::new(resp.token, layer_context).into(),
                }),
        )
    }
}

impl<Inner, LR> ResourceStatusTokenResolver for LayeredResourceStatusTokenResolver<Inner, LR>
where
    Inner: ResourceStatusTokenResolver,
    LR: Repo<
        StSpace = <Inner::Repo as Repo>::StSpace,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            <Inner::Repo as Repo>::ResourceStatusTokenTypes,
            LR,
        >,
    >,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
}
