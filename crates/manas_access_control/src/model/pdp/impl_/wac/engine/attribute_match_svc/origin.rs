//! I define `acl:origin`  attribute match service as defined by wac specification.
//!

use std::{borrow::Borrow, fmt::Debug, task::Poll};

use acp::attribute_match_svc::AttributeMatchRequest;
use dyn_problem::{ProbFuture, Problem};
use rdf_utils::model::{description::DescriptionExt, graph::InfallibleGraph};
use rdf_vocabularies::ns;
use sophia_api::term::Term;
use tower::Service;

/// An [`AttributeMatchService`](acp::attribute_match_svc::AttributeMatchService) that resolves match
/// for `acl::origin` attribute.
///
#[ghost::phantom]
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct OriginMatchService<T, G, WG>;

impl<T, G, WG> Service<AttributeMatchRequest<T, G, WG>> for OriginMatchService<T, G, WG>
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
        Box::pin(futures::future::ready(Ok(Self::match_origin(req))))
    }
}

impl<T, G, WG> OriginMatchService<T, G, WG>
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    fn match_origin(req: AttributeMatchRequest<T, G, WG>) -> bool {
        let AttributeMatchRequest {
            value: m_origin,
            context,
        } = req;

        if context.has_any_with(&ns::acl::origin, &m_origin) {
            return true;
        }

        false
    }
}
