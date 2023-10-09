use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use tower::Service;

use crate::{
    layer::LayeredRepo,
    service::resource_operator::{
        common::impl_::DelegatingOperator,
        deleter::{ResourceDeleteRequest, ResourceDeleteResponse, ResourceDeleter},
    },
    Repo,
};

impl<Inner, LR> Service<ResourceDeleteRequest<LR>> for DelegatingOperator<Inner, LR>
where
    Inner: ResourceDeleter,
    LR: LayeredRepo<Inner::Repo>,
    LR::Credentials: Into<<Inner::Repo as Repo>::Credentials>,
{
    type Response = ResourceDeleteResponse<LR>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ResourceDeleteRequest<LR>) -> Self::Future {
        Box::pin(
            self.inner
                .call(req.unlayer_tokens())
                .map_ok(|resp: ResourceDeleteResponse<_>| resp.map_repo()),
        )
    }
}

impl<Inner, LR> ResourceDeleter for DelegatingOperator<Inner, LR>
where
    Inner: ResourceDeleter,
    LR: LayeredRepo<Inner::Repo>,
    LR::Credentials: Into<<Inner::Repo as Repo>::Credentials>,
{
    type Repo = LR;
}
