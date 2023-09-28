use dyn_problem::{ProbFuture, Problem};
use tower::Service;

use crate::{
    context::LayeredRepoContext,
    service::resource_operator::{
        common::{
            impl_::DelegatingOperator,
            status_token::impl_::layered::LayeredResourceStatusTokenTypes,
        },
        updater::{ResourceUpdateRequest, ResourceUpdateResponse, ResourceUpdater},
    },
    Repo,
};

impl<Inner, LR> Service<ResourceUpdateRequest<LR>> for DelegatingOperator<Inner, LR>
where
    Inner: ResourceUpdater,
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
    type Response = ResourceUpdateResponse;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ResourceUpdateRequest<LR>) -> Self::Future {
        Box::pin(self.inner.call(req.unlayer_tokens()))
    }
}

impl<Inner, LR> ResourceUpdater for DelegatingOperator<Inner, LR>
where
    Inner: ResourceUpdater,
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
