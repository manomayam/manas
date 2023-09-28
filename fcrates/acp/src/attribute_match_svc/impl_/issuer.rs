//! I define `issuer`  attribute match service as defined by acp specification.
//!

use std::{borrow::Borrow, fmt::Debug, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use rdf_utils::model::{description::DescriptionExt, graph::InfallibleGraph};
use rdf_vocabularies::ns;
use sophia_api::term::Term;
use tower::Service;

use crate::attribute_match_svc::AttributeMatchRequest;

/// An [`AttributeMatchService`](super::super::AttributeMatchService) that resolves match
/// for `acp::issuer` attribute.
///
#[ghost::phantom]
#[allow(missing_docs)]
#[derive(Debug, Clone, Default)]
pub struct IssuerMatchService<T, G, WG>;

impl<T, G, WG> Service<AttributeMatchRequest<T, G, WG>> for IssuerMatchService<T, G, WG>
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
        Box::pin(futures::future::ready(Ok(Self::match_issuer(req))))
    }
}

impl<T, G, WG> IssuerMatchService<T, G, WG>
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    fn match_issuer(req: AttributeMatchRequest<T, G, WG>) -> bool {
        let AttributeMatchRequest {
            value: m_issuer_id,
            context,
        } = req;

        // > In a Matcher, issuer attributes using the Public Issuer
        // > named individual MUST match all Contexts.
        if Term::eq(&m_issuer_id, ns::acp::PublicIssuer) {
            return true;
        }

        // > In a Matcher, client attributes using the AuthenticatedIssuer
        // > named individual MUST match Contexts that contain an issuer.
        if Term::eq(&m_issuer_id, ns::acp::AuthenticatedIssuer) && context.has_any(&ns::acp::issuer)
        {
            return true;
        }

        if context.has_any_with(&ns::acp::issuer, &m_issuer_id) {
            return true;
        }

        false
    }
}
