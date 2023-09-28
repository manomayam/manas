//! I define an implementation of [`ResourceReader`](super::ResourceReader)
//! for unsupported services.

use dyn_problem::{ProbFuture, Problem};
use tower::Service;

use crate::{
    service::resource_operator::{
        common::{impl_::UnsupportedOperator, problem::UNSUPPORTED_OPERATION},
        reader::{
            message::{ResourceReadRequest, ResourceReadResponse},
            FlexibleResourceReader, ResourceReader,
        },
    },
    Repo,
};

impl<R: Repo> Service<ResourceReadRequest<R>> for UnsupportedOperator<R> {
    type Response = ResourceReadResponse<R, R::Representation>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, _req: ResourceReadRequest<R>) -> Self::Future {
        Box::pin(futures::future::ready(Err(UNSUPPORTED_OPERATION
            .new_problem_builder()
            .message("Resource read is not supported by this repo.")
            .finish())))
    }
}

impl<R: Repo> FlexibleResourceReader<R, R::Representation> for UnsupportedOperator<R> {}

impl<R: Repo> ResourceReader for UnsupportedOperator<R> {
    type Repo = R;
}
