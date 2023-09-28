//! I define `vc`  attribute match service as defined by acp specification.
//!

use std::{borrow::Borrow, fmt::Debug, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use rdf_utils::model::{description::DescriptionExt, graph::InfallibleGraph};
use rdf_vocabularies::ns;
use sophia_api::term::Term;
use tower::Service;

use crate::attribute_match_svc::AttributeMatchRequest;

/// An [`AttributeMatchService`](super::super::AttributeMatchService) that resolves match
/// for `acp::vc` attribute.
///
#[ghost::phantom]
#[allow(missing_docs)]
#[derive(Debug, Clone, Default)]
pub struct VcMatchService<T, G, WG>;

impl<T, G, WG> Service<AttributeMatchRequest<T, G, WG>> for VcMatchService<T, G, WG>
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    type Response = bool;

    type Error = Problem;

    type Future = ProbFuture<'static, bool>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: AttributeMatchRequest<T, G, WG>) -> Self::Future {
        Box::pin(futures::future::ready(Ok(Self::match_vc(req))))
    }
}

impl<T, G, WG> VcMatchService<T, G, WG>
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    fn match_vc(req: AttributeMatchRequest<T, G, WG>) -> bool {
        let AttributeMatchRequest {
            value: m_vc_id,
            context,
        } = req;

        if context.has_any_with(&ns::acp::vc, &m_vc_id) {
            return true;
        }

        false
    }
}
