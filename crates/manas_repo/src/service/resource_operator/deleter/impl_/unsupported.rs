//! I define an implementation of [`ResourceDeleter`](super::ResourceDeleter)
//! for unsupported services.
//!

use dyn_problem::{ProbFuture, Problem};
use tower::Service;

use crate::{
    service::resource_operator::{
        common::{impl_::UnsupportedOperator, problem::UNSUPPORTED_OPERATION},
        deleter::{ResourceDeleteRequest, ResourceDeleteResponse, ResourceDeleter},
    },
    Repo,
};

impl<R: Repo> Service<ResourceDeleteRequest<R>> for UnsupportedOperator<R> {
    type Response = ResourceDeleteResponse<R>;

    type Error = Problem;

    type Future = ProbFuture<'static, ResourceDeleteResponse<R>>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, _req: ResourceDeleteRequest<R>) -> Self::Future {
        Box::pin(futures::future::ready(Err(UNSUPPORTED_OPERATION
            .new_problem_builder()
            .message("Resource updation is not supported by this repo.")
            .finish())))
    }
}

impl<R: Repo> ResourceDeleter for UnsupportedOperator<R> {
    type Repo = R;
}
