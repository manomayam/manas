use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use tower::Service;

use crate::{
    context::LayeredRepoContext,
    service::resource_operator::{
        common::{
            impl_::DelegatingOperator,
            status_token::impl_::layered::LayeredResourceStatusTokenTypes,
        },
        creator::{ResourceCreateRequest, ResourceCreateResponse, ResourceCreator},
    },
    Repo,
};

impl<Inner, LR> Service<ResourceCreateRequest<LR>> for DelegatingOperator<Inner, LR>
where
    Inner: ResourceCreator,
    LR: Repo<
        StSpace = <Inner::Repo as Repo>::StSpace,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            <Inner::Repo as Repo>::ResourceStatusTokenTypes,
            LR,
        >,
    >,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
    LR::Representation: Into<<Inner::Repo as Repo>::Representation>,
    LR::RepPatcher: Into<<Inner::Repo as Repo>::RepPatcher>,
    LR::Credentials: Into<<Inner::Repo as Repo>::Credentials>,
{
    type Response = ResourceCreateResponse<LR>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ResourceCreateRequest<LR>) -> Self::Future {
        Box::pin(
            self.inner
                .call(req.unlayer_tokens())
                .map_ok(|resp: ResourceCreateResponse<_>| resp.map_repo()),
        )
    }
}

impl<Inner, LR> ResourceCreator for DelegatingOperator<Inner, LR>
where
    Inner: ResourceCreator,
    LR: Repo<
        StSpace = <Inner::Repo as Repo>::StSpace,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            <Inner::Repo as Repo>::ResourceStatusTokenTypes,
            LR,
        >,
    >,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
    LR::Representation: Into<<Inner::Repo as Repo>::Representation>,
    LR::RepPatcher: Into<<Inner::Repo as Repo>::RepPatcher>,
    LR::Credentials: Into<<Inner::Repo as Repo>::Credentials>,
{
    type Repo = LR;
}
