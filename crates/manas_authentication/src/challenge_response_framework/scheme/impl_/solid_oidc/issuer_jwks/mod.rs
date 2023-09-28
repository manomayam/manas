//! I define utils to resolve configured jwks of oidc issuers.
//!

use std::{fmt::Debug, sync::Arc};

use futures::future::BoxFuture;
use http_uri::invariant::AbsoluteHttpUri;
use picky::jose::jwk::JwkSet;
use tracing::error;

pub mod impl_;

/// A trait for oidc issuer jwks resolvers.
pub trait OidcIssuerJwksResolver: Debug + Send + 'static + Sync {
    /// Resolve jwks of oidc issuer.
    fn resolve(
        &self,
        iss: AbsoluteHttpUri,
    ) -> BoxFuture<'_, Result<JwkSet, Arc<OidcIssuerJwksResolutionError>>>;
}

/// An error type for representing errors in oidc issuer's jwks resolution.
#[derive(Debug, thiserror::Error)]
pub enum OidcIssuerJwksResolutionError {
    /// Unknown io error.
    #[error("Unknown io error.")]
    UnknownIoError(#[from] reqwest::Error),

    /// Invalid oidc issuer config response.
    #[error("Invalid oidc issuer config response.")]
    InvalidOidcIssuerConfigResponse,

    /// Invalid jwks deref response.
    #[error("Invalid jwks deref response.")]
    InvalidJwksDerefResponse,
}
