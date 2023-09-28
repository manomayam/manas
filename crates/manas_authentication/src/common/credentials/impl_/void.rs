//! I define a void implementation of the [`RequestCredentials`].
//!

use serde::Serialize;

use super::basic::{BasicAgentCredentials, BasicClientCredentials, BasicIssuerCredentials};
use crate::common::credentials::RequestCredentials;

/// A struct to represent void credentials.
#[derive(Debug, Clone, Default, Serialize)]
pub struct VoidCredentials;

impl RequestCredentials for VoidCredentials {
    type AgentCredentials = BasicAgentCredentials;

    type ClientCredentials = BasicClientCredentials;

    type IssuerCredentials = BasicIssuerCredentials;

    #[inline]
    fn of_agent(&self) -> Option<&Self::AgentCredentials> {
        None
    }

    #[inline]
    fn of_client(&self) -> Option<&Self::ClientCredentials> {
        None
    }

    #[inline]
    fn of_issuer(&self) -> Option<&Self::IssuerCredentials> {
        None
    }
}
