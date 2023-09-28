//! I define `agent`  attribute match service as defined by acp specification.
//!

use std::{borrow::Borrow, fmt::Debug, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use rdf_utils::model::{description::DescriptionExt, graph::InfallibleGraph};
use rdf_vocabularies::ns;
use sophia_api::term::Term;
use tower::Service;

use crate::attribute_match_svc::AttributeMatchRequest;

/// An [`AttributeMatchService`](super::super::AttributeMatchService) that resolves match
/// for `acp::agent` attribute.
///
#[ghost::phantom]
#[allow(missing_docs)]
#[derive(Debug, Clone, Default)]
pub struct AgentMatchService<T, G, WG>;

impl<T, G, WG> Service<AttributeMatchRequest<T, G, WG>> for AgentMatchService<T, G, WG>
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
        Box::pin(futures::future::ready(Ok(Self::match_agent(req))))
    }
}

impl<T, G, WG> AgentMatchService<T, G, WG>
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    fn match_agent(req: AttributeMatchRequest<T, G, WG>) -> bool {
        let AttributeMatchRequest {
            value: m_agent_id,
            context,
        } = req;

        // > In a Matcher, agent attributes using the Public Agent
        // > named individual MUST match all Contexts.
        if Term::eq(&m_agent_id, ns::acp::PublicAgent) {
            return true;
        }

        // > In a Matcher, agent attributes using the Authenticated Agent
        // > named individual MUST match Contexts that contain an agent.
        if Term::eq(&m_agent_id, ns::acp::AuthenticatedAgent) && context.has_any(&ns::acp::agent) {
            return true;
        }

        // > In a Matcher, agent attributes using the Creator Agent
        // > named individual MUST match Contexts where a defined creator
        //> matches the defined agent.
        if Term::eq(&m_agent_id, ns::acp::CreatorAgent)
            && context.has_common(&ns::acp::creator, &ns::acp::agent)
        {
            return true;
        }

        // > In a Matcher, agent attributes using the Owner Agent
        // > named individual MUST match Contexts where a defined
        // > owner matches the defined agent.
        if Term::eq(&m_agent_id, ns::acp::OwnerAgent)
            && context.has_common(&ns::acp::owner, &ns::acp::agent)
        {
            return true;
        }

        if context.has_any_with(&ns::acp::agent, &m_agent_id) {
            return true;
        }

        false
    }
}
