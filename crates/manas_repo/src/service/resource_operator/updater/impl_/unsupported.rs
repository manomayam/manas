//! I define an implementation of [`ResourceUpdater`](super::ResourceUpdater)
//! for unsupported services.
//!

use dyn_problem::{ProbFuture, Problem};
use tower::Service;

use crate::{
    service::resource_operator::{
        common::{impl_::UnsupportedOperator, problem::UNSUPPORTED_OPERATION},
        updater::{ResourceUpdateRequest, ResourceUpdateResponse, ResourceUpdater},
    },
    Repo,
};

impl<R: Repo> Service<ResourceUpdateRequest<R>> for UnsupportedOperator<R> {
    type Response = ResourceUpdateResponse;

    type Error = Problem;

    type Future = ProbFuture<'static, ResourceUpdateResponse>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, _req: ResourceUpdateRequest<R>) -> Self::Future {
        Box::pin(futures::future::ready(Err(UNSUPPORTED_OPERATION
            .new_problem_builder()
            .message("Resource update operation is not supported by this repo.")
            .finish())))
    }
}

impl<R: Repo> ResourceUpdater for UnsupportedOperator<R> {
    type Repo = R;
}
