//! I define `acl:agentClass`  attribute match service as
//! defined by wac specification.
//!

use std::{borrow::Borrow, fmt::Debug, task::Poll};

use acp::attribute_match_svc::AttributeMatchRequest;
use dyn_problem::{ProbFuture, Problem};
use rdf_utils::model::{description::DescriptionExt, graph::InfallibleGraph};
use rdf_vocabularies::ns;
use sophia_api::term::Term;
use tower::Service;

/// An [`AttributeMatchService`](acp::attribute_match_svc::AttributeMatchService) that resolves match
/// for `acl::agentClass` attribute.
///
#[ghost::phantom]
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct AgentClassMatchService<T, G, WG>;

impl<T, G, WG> Service<AttributeMatchRequest<T, G, WG>> for AgentClassMatchService<T, G, WG>
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
        Box::pin(futures::future::ready(Ok(Self::match_agent_class(req))))
    }
}

impl<T, G, WG> AgentClassMatchService<T, G, WG>
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    fn match_agent_class(req: AttributeMatchRequest<T, G, WG>) -> bool {
        let AttributeMatchRequest {
            value: m_agent_class,
            context,
        } = req;

        // > foaf:Agent
        // >    Allows access to any agent, i.e., the public.
        if Term::eq(&m_agent_class, ns::foaf::Agent) {
            return true;
        }

        // > acl:AuthenticatedAgent
        // >     Allows access to any authenticated agent.
        if Term::eq(&m_agent_class, ns::acl::AuthenticatedAgent) && context.has_any(&ns::acp::agent)
        {
            return true;
        }

        // TODO agentClass extendability if required.
        false
    }
}
