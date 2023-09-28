//! I provide an implementation of [`RepPatcherResolver`] that
//! delegates to an inner resolver.
//!

use std::task::Poll;

use dyn_problem::{ProbFuture, Problem};
use manas_http::representation::impl_::binary::BinaryRepresentation;
use tower::Service;

use crate::{
    context::{impl_::DelegatedRepoContextual, LayeredRepoContext},
    service::patcher_resolver::RepPatcherResolver,
    Repo,
};

/// an implementation of [`RepPatcherResolver`] that
/// delegates to an inner repo rep patcher resolver.
pub type DelegatedRepPatcherResolver<Inner, LR> = DelegatedRepoContextual<Inner, LR>;

impl<Inner, LR> Service<BinaryRepresentation> for DelegatedRepPatcherResolver<Inner, LR>
where
    Inner: RepPatcherResolver,
    LR: Repo<RepPatcher = <Inner::Repo as Repo>::RepPatcher>,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
    type Response = LR::RepPatcher;

    type Error = Problem;

    type Future = ProbFuture<'static, LR::RepPatcher>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<BinaryRepresentation>::poll_ready(&mut self.inner, cx)
    }

    #[inline]
    fn call(&mut self, req: BinaryRepresentation) -> Self::Future {
        Service::<BinaryRepresentation>::call(&mut self.inner, req)
    }
}

impl<Inner, LR> RepPatcherResolver for DelegatedRepoContextual<Inner, LR>
where
    Inner: RepPatcherResolver,
    LR: Repo<RepPatcher = <Inner::Repo as Repo>::RepPatcher>,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
}
