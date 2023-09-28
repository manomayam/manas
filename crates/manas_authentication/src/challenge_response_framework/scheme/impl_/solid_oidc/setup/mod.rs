//! I define types and traits for representing setup for [`SolidOidcDpopScheme`](super::SolidOidcDpopScheme).
//!

use std::fmt::Debug;

use http_uri::security::transport_policy::SecureTransportPolicy;

use super::{issuer_jwks::OidcIssuerJwksResolver, trusted_issuers::WebIdTrustedIssuersResolver};

pub mod impl_;

/// Trait for setup of [`SolidOidcDpopScheme`](super::SolidOidcDpopScheme).
pub trait SolidOidcDpopSchemeSetup: Debug + Send + 'static {
    /// Type of secure transport policy.
    type SecureTransportPolicy: SecureTransportPolicy;

    /// Type of oidc issuer jwks resolver.
    type IssuerJwksResolver: OidcIssuerJwksResolver;

    /// Type of webid trusted issuers resolver.
    type WebIdIssuersResolver: WebIdTrustedIssuersResolver;
}
