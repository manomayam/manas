//! I define default implementation of the [`OidcIssuerJwksResolver`].
//!

use std::{fmt::Debug, sync::Arc, time::Duration};

use futures::{future::BoxFuture, TryFutureExt};
use http_uri::invariant::AbsoluteHttpUri;
use moka::future::{Cache, CacheBuilder};
use picky::jose::jwk::JwkSet;
use reqwest::{header::ACCEPT, Client};
use tracing::error;

use crate::challenge_response_framework::scheme::impl_::solid_oidc::{
    issuer_jwks::{OidcIssuerJwksResolutionError, OidcIssuerJwksResolver},
    CacheConfig,
};

/// A struct for resolving jwks of a oidc token issuer.
#[derive(Debug, Clone)]
pub struct DefaultOidcIssuerJwksResolver {
    /// Http client.
    client: Client,

    /// Cache.
    cache: Cache<AbsoluteHttpUri, JwkSet>,
}

impl OidcIssuerJwksResolver for DefaultOidcIssuerJwksResolver {
    #[tracing::instrument(skip_all, name = "DefaultOidcIssuerJwksResolver::resolve", fields(iss))]
    fn resolve<'s>(
        &self,
        iss: AbsoluteHttpUri,
    ) -> BoxFuture<'_, Result<JwkSet, Arc<OidcIssuerJwksResolutionError>>> {
        Box::pin(
            self.cache
                .try_get_with(iss.clone(), self.resolve_fresh(iss)),
        )
    }
}

impl Default for DefaultOidcIssuerJwksResolver {
    fn default() -> Self {
        Self::new(CacheConfig {
            max_capacity: 5000,
            // 5 minutes by default.
            time_to_live: Duration::from_secs(300),
        })
    }
}

impl DefaultOidcIssuerJwksResolver {
    /// Create a new [`OidcIssuerJwksResolver`].
    pub fn new(cache_config: CacheConfig) -> Self {
        let client = Client::new();

        let cache = CacheBuilder::new(cache_config.max_capacity)
            .time_to_live(cache_config.time_to_live)
            .build();

        Self { client, cache }
    }

    /// Resolve jwks of the oidc issuer without cache.
    async fn resolve_fresh(
        &self,
        iss: AbsoluteHttpUri,
    ) -> Result<JwkSet, OidcIssuerJwksResolutionError> {
        // Resolve jwks uri.
        let jwks_uri = self
            .resolve_jwks_uri(&iss)
            .inspect_err(|_| error!("Error in resolving jwks uri of the issuer."))
            .await?;

        // Send dereference request.
        let resp = self
            .client
            .get(jwks_uri.as_str())
            .header(ACCEPT, mime::APPLICATION_JSON.essence_str())
            .send()
            .map_err(|e| {
                error!("Unknown io error in dereferencing jwks. Error:\n {}", e);
                OidcIssuerJwksResolutionError::UnknownIoError(e)
            })
            .await?;

        if !resp.status().is_success() {
            error!("Error in dereferencing jwks. Status: {}", resp.status());
            return Err(OidcIssuerJwksResolutionError::InvalidJwksDerefResponse);
        }

        // Deserialize body.
        resp.json()
            .map_err(|_| {
                error!("Invalid jwks dereference response.");
                OidcIssuerJwksResolutionError::InvalidJwksDerefResponse
            })
            .await
    }

    /// Resolve jwks uri for the issuer.
    async fn resolve_jwks_uri(
        &self,
        iss: &AbsoluteHttpUri,
    ) -> Result<AbsoluteHttpUri, OidcIssuerJwksResolutionError> {
        // Get issuer config.
        let issuer_config = self
            .deref_issuer_config(iss)
            .inspect_err(|_| {
                error!("Error in dereferencing issuer configuration");
            })
            .await?;

        // Return `jwks_uri`.
        Ok(issuer_config.jwks_uri)
    }

    /// Dereference issuer config.
    async fn deref_issuer_config(
        &self,
        iss: &AbsoluteHttpUri,
    ) -> Result<RequiredOidcIssuerConfig, OidcIssuerJwksResolutionError> {
        // Construct issuer config uri.
        let config_uri = Self::get_wellknown_openid_config_uri(iss);

        // Make http request to config resource.
        let resp = self
            .client
            .get(config_uri.as_str())
            .header(ACCEPT, mime::APPLICATION_JSON.essence_str())
            .send()
            .map_err(|e| {
                error!(
                    "Unknown io error in getting oidc issuer config. Error:\n {}",
                    e
                );
                OidcIssuerJwksResolutionError::UnknownIoError(e)
            })
            .await?;

        if !resp.status().is_success() {
            error!(
                "Issuer openid config deref error. Status: {}",
                resp.status()
            );
            return Err(OidcIssuerJwksResolutionError::InvalidOidcIssuerConfigResponse);
        }

        // Deserialize body json.
        resp.json()
            .map_err(|_| OidcIssuerJwksResolutionError::InvalidOidcIssuerConfigResponse)
            .await
    }

    /// Get `.well-known` openid configuration uri for given oidc issuer.
    /// > OpenID Providers supporting Discovery MUST make a JSON document available
    /// at the path formed by concatenating the string /.well-known/openid-configuration to the Issuer.
    fn get_wellknown_openid_config_uri(iss: &AbsoluteHttpUri) -> AbsoluteHttpUri {
        AbsoluteHttpUri::try_new_from(
            format!(
                "{}/.well-known/openid-configuration",
                iss.as_str().trim_end_matches('/')
            )
            .as_str(),
        )
        .expect("Must be an absolute http uri.")
    }
}

/// A helper struct for deserializing required openid issuer config.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RequiredOidcIssuerConfig {
    pub jwks_uri: AbsoluteHttpUri,
}
