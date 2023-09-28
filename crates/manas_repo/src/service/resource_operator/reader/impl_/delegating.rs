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
        reader::{
            FlexibleResourceReader, ResourceReadRequest, ResourceReadResponse, ResourceReader,
        },
    },
    Repo,
};

impl<Inner, LR> Service<ResourceReadRequest<LR>> for DelegatingOperator<Inner, LR>
where
    Inner: ResourceReader,
    LR: Repo<
        StSpace = <Inner::Repo as Repo>::StSpace,
        Representation = <Inner::Repo as Repo>::Representation,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            <Inner::Repo as Repo>::ResourceStatusTokenTypes,
            LR,
        >,
    >,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
    LR::Credentials: Into<<Inner::Repo as Repo>::Credentials>,
{
    type Response = ResourceReadResponse<LR, LR::Representation>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ResourceReadRequest<LR>) -> Self::Future {
        let layer_context = req.tokens.res_token.layer_context.clone();
        Box::pin(
            self.inner
                .call(req.unlayer_tokens())
                .map_ok(|resp: ResourceReadResponse<_, _>| resp.layer_tokens(layer_context)),
        )
    }
}

impl<Inner, LR> FlexibleResourceReader<LR, LR::Representation> for DelegatingOperator<Inner, LR>
where
    Inner: ResourceReader,
    LR: Repo<
        StSpace = <Inner::Repo as Repo>::StSpace,
        Representation = <Inner::Repo as Repo>::Representation,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            <Inner::Repo as Repo>::ResourceStatusTokenTypes,
            LR,
        >,
    >,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
    LR::Credentials: Into<<Inner::Repo as Repo>::Credentials>,
{
}

impl<Inner, LR> ResourceReader for DelegatingOperator<Inner, LR>
where
    Inner: ResourceReader,
    LR: Repo<
        StSpace = <Inner::Repo as Repo>::StSpace,
        Representation = <Inner::Repo as Repo>::Representation,
        ResourceStatusTokenTypes = LayeredResourceStatusTokenTypes<
            <Inner::Repo as Repo>::ResourceStatusTokenTypes,
            LR,
        >,
    >,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
    LR::Credentials: Into<<Inner::Repo as Repo>::Credentials>,
{
    type Repo = LR;
}
