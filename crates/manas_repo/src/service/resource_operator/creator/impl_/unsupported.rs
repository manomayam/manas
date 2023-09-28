//! I define an implementation of [`ResourceCreator`](super::ResourceCreator)
//! for unsupported services.
//!

use dyn_problem::{ProbFuture, Problem};
use tower::Service;

use crate::{
    service::resource_operator::{
        common::{impl_::UnsupportedOperator, problem::UNSUPPORTED_OPERATION},
        creator::{ResourceCreateRequest, ResourceCreateResponse, ResourceCreator},
    },
    Repo,
};

impl<R: Repo> Service<ResourceCreateRequest<R>> for UnsupportedOperator<R> {
    type Response = ResourceCreateResponse<R>;

    type Error = Problem;

    type Future = ProbFuture<'static, ResourceCreateResponse<R>>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, _req: ResourceCreateRequest<R>) -> Self::Future {
        Box::pin(futures::future::ready(Err(UNSUPPORTED_OPERATION
            .new_problem_builder()
            .message("Resource creation is not supported by this repo.")
            .finish())))
    }
}

impl<R: Repo> ResourceCreator for UnsupportedOperator<R> {
    type Repo = R;
}
