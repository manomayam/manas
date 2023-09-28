//! I provide an implementation of [`ResourceReader`] for
//! [`DerivedContentNegotiatingRepo`](crate::dconneging::DerivedContentNegotiatingRepo).
//!

use std::marker::PhantomData;

use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use manas_repo::{
    service::resource_operator::reader::{
        FlexibleResourceReader, ResourceReadRequest, ResourceReadResponse, ResourceReader,
    },
    Repo, RepoResourceReader,
};
use tower::{Service, ServiceExt};

use crate::dconneging::{conneg_layer::DerivedContentNegotiationLayer, MRepo};

/// An implementation of [`ResourceReader`] for [`DerivedContentNegotiatingRepo`](crate::dconneging::DerivedContentNegotiatingRepo).
#[derive(Debug)]
pub struct DerivedContentNegotiatingResourceReader<IR: Repo, CNL> {
    inner: RepoResourceReader<IR>,
    _phantom: PhantomData<fn(CNL)>,
}

impl<IR: Repo, CNL> Default for DerivedContentNegotiatingResourceReader<IR, CNL> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<IR: Repo, CNL> Clone for DerivedContentNegotiatingResourceReader<IR, CNL> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<IR, CNL> Service<ResourceReadRequest<MRepo<IR, CNL>>>
    for DerivedContentNegotiatingResourceReader<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    type Response = ResourceReadResponse<MRepo<IR, CNL>, IR::Representation>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "DerivedContentNegotiatingResourceReader::call")]
    fn call(&mut self, req: ResourceReadRequest<MRepo<IR, CNL>>) -> Self::Future {
        let layer_context = req.tokens.res_token.layer_context.clone();

        let mut w_inner_service =
            CNL::new(layer_context.dconneg_layer_config.clone()).layer(self.inner.clone());

        let inner_req = req.unlayer_tokens();

        Box::pin(async move {
            w_inner_service
                .ready()
                .and_then(|svc| {
                    svc.call(inner_req)
                        .map_ok(|resp| resp.layer_tokens(layer_context))
                })
                .await
        })
    }
}

impl<IR, CNL> FlexibleResourceReader<MRepo<IR, CNL>, IR::Representation>
    for DerivedContentNegotiatingResourceReader<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
}

impl<IR, CNL> ResourceReader for DerivedContentNegotiatingResourceReader<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    type Repo = MRepo<IR, CNL>;
}
