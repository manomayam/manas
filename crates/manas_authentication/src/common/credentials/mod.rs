//! I define traits and types to represent request resolved credentials.
//!

use std::fmt::Debug;

use http_uri::invariant::AbsoluteHttpUri;
use serde::Serialize;
use webid::WebId;

pub mod impl_;

/// A trait for representing credentials of the request agent.
pub trait AgentCredentials: Debug + Clone + Send + Sync + 'static {
    /// Get the webid of the agent.
    fn webid(&self) -> &WebId;
}

/// A type for defining client id.
pub type ClientId = String;

/// A trait for defining client credentials.
pub trait ClientCredentials: Debug + Clone + Send + Sync + 'static {
    /// Get the id of the client.
    fn client_id(&self) -> &ClientId;

    /// Get the optional webid of the client.
    fn client_web_id(&self) -> Option<&WebId>;
}

/// A trait for defining issuer credentials.
pub trait IssuerCredentials: Debug + Clone + Send + Sync + 'static {
    /// Get the uri of the issuer.
    fn uri(&self) -> &AbsoluteHttpUri;
}

/// A trait to represent request credentials.
pub trait RequestCredentials: Debug + Default + Clone + Send + Sync + 'static + Serialize {
    /// Type of the agent credentials.
    type AgentCredentials: AgentCredentials;

    /// Type of the client credentials.
    type ClientCredentials: ClientCredentials;

    /// Type of the issuer credentials.
    type IssuerCredentials: IssuerCredentials;

    /// Get agent credentials.
    fn of_agent(&self) -> Option<&Self::AgentCredentials>;

    /// Get client credentials.
    fn of_client(&self) -> Option<&Self::ClientCredentials>;

    /// Get issuer credentials.
    fn of_issuer(&self) -> Option<&Self::IssuerCredentials>;
}

#[cfg(feature = "creds-context")]
pub use to_context_def::ToContext;

#[cfg(feature = "creds-context")]
mod to_context_def {
    use std::ops::Deref;

    use acp::model::context::{DContext, HContext};
    use rdf_utils::model::{
        description::{DescriptionExt, SimpleDescription},
        graph::InfallibleMutableGraph,
        term::ArcTerm,
    };
    use rdf_vocabularies::ns;

    use super::RequestCredentials;
    use crate::common::credentials::{AgentCredentials, ClientCredentials, IssuerCredentials};

    /// A trait for request credentials that can convert to acp
    /// access context.
    pub trait ToContext: RequestCredentials {
        /// Convert the request credentials to acp access
        /// context.
        fn to_context<G: InfallibleMutableGraph + Default>(
            &self,
            h_context: HContext<ArcTerm>,
        ) -> DContext<G, G> {
            let mut context = DContext::new(h_context, G::default());

            // Add agent context.
            if let Some(agent_creds) = self.of_agent() {
                context.add(&ns::acp::agent, agent_creds.webid());
                context.add(&ns::acl::agent, agent_creds.webid());
            }

            // Add client context.
            if let Some(client_web_id) = self
                .of_client()
                .and_then(|of_client| of_client.client_web_id())
            {
                context.add(&ns::acp::client, client_web_id);
            }

            // Add issuer context.
            if let Some(issuer_creds) = self.of_issuer() {
                context.add(&ns::acp::issuer, issuer_creds.uri().deref());
            }

            context
        }
    }
}
