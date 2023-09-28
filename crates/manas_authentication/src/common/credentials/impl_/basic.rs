//! I define a basic implementation of the [`RequestCredentials`].
//!

use http_uri::invariant::AbsoluteHttpUri;
use serde::Serialize;
use webid::WebId;

use super::void::VoidCredentials;
use crate::common::credentials::{
    AgentCredentials, ClientCredentials, ClientId, IssuerCredentials, RequestCredentials,
};

/// A basic implementation of [`AgentCredentials`].
#[derive(Debug, Clone, Serialize)]
pub struct BasicAgentCredentials {
    /// Webid of the agent.
    pub webid: WebId,
}

impl AgentCredentials for BasicAgentCredentials {
    #[inline]
    fn webid(&self) -> &WebId {
        &self.webid
    }
}

/// A basic implementation of [`ClientCredentials`].
#[derive(Debug, Clone, Serialize)]
pub struct BasicClientCredentials {
    /// Client_id of the client.
    pub client_id: ClientId,

    /// Webid of the client.
    pub client_web_id: Option<WebId>,
}

impl ClientCredentials for BasicClientCredentials {
    #[inline]
    fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    #[inline]
    fn client_web_id(&self) -> Option<&WebId> {
        self.client_web_id.as_ref()
    }
}

/// A basic implementation of [`IssuerCredentials`].
#[derive(Debug, Clone, Serialize)]
pub struct BasicIssuerCredentials {
    /// Uri of the issuer.
    pub uri: AbsoluteHttpUri,
}

impl IssuerCredentials for BasicIssuerCredentials {
    #[inline]
    fn uri(&self) -> &AbsoluteHttpUri {
        &self.uri
    }
}

/// A basic implementation of [`RequestCredentials`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct BasicRequestCredentials {
    /// Agent credentials.
    pub of_agent: Option<BasicAgentCredentials>,

    /// Client credentials.
    pub of_client: Option<BasicClientCredentials>,

    /// Issuer credentials.
    pub of_issuer: Option<BasicIssuerCredentials>,
}

impl RequestCredentials for BasicRequestCredentials {
    type AgentCredentials = BasicAgentCredentials;

    type ClientCredentials = BasicClientCredentials;

    type IssuerCredentials = BasicIssuerCredentials;

    #[inline]
    fn of_agent(&self) -> Option<&Self::AgentCredentials> {
        self.of_agent.as_ref()
    }

    #[inline]
    fn of_client(&self) -> Option<&Self::ClientCredentials> {
        self.of_client.as_ref()
    }

    #[inline]
    fn of_issuer(&self) -> Option<&Self::IssuerCredentials> {
        self.of_issuer.as_ref()
    }
}

#[cfg(feature = "creds-context")]
impl super::super::ToContext for BasicRequestCredentials {}

impl From<BasicRequestCredentials> for VoidCredentials {
    #[inline]
    fn from(_value: BasicRequestCredentials) -> Self {
        Self {}
    }
}
