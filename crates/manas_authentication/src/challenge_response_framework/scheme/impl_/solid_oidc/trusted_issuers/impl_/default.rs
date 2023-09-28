//! I define default implementation of [`WebIdTrustedIssuersResolver`].
//!

use std::{collections::HashSet, fmt::Debug, sync::Arc, time::Duration};

use futures::{future::BoxFuture, TryFutureExt};
use http_uri::invariant::AbsoluteHttpUri;
use moka::future::{Cache, CacheBuilder};
use rdf_vocabularies::ns;
use sophia_api::{
    graph::Graph,
    term::{matcher::Any, SimpleTerm, Term},
    triple::Triple,
};
use tracing::error;
use unwrap_infallible::UnwrapInfallible;
use webid::{profile_req_agent::WebIdProfileReqAgent, WebId};

use crate::challenge_response_framework::scheme::impl_::solid_oidc::{
    trusted_issuers::{WebIdTrustedIssuersResolutionError, WebIdTrustedIssuersResolver},
    CacheConfig,
};

/// A default implementation of [`WebIdTrustedIssuersResolver`].
#[derive(Debug, Clone)]
pub struct DefaultWebIdTrustedIssuersResolver {
    /// Webid profile request agent.
    profile_req_agent: WebIdProfileReqAgent,

    /// Cache.
    cache: Cache<WebId, Vec<AbsoluteHttpUri>>,
}

impl WebIdTrustedIssuersResolver for DefaultWebIdTrustedIssuersResolver {
    #[tracing::instrument(
        skip_all,
        name = "DefaultWebIdTrustedIssuersResolver::resolve",
        fields(webid)
    )]
    fn resolve(
        &self,
        webid: WebId,
    ) -> BoxFuture<'_, Result<Vec<AbsoluteHttpUri>, Arc<WebIdTrustedIssuersResolutionError>>> {
        Box::pin(
            self.cache
                .try_get_with(webid.clone(), self.resolve_fresh(webid)),
        )
    }
}

impl Default for DefaultWebIdTrustedIssuersResolver {
    fn default() -> Self {
        Self::new(CacheConfig {
            max_capacity: 5000,
            // 5 minutes by default.
            time_to_live: Duration::from_secs(300),
        })
    }
}

impl DefaultWebIdTrustedIssuersResolver {
    /// Create a new [`WebIdTrustedIssuersResolver`].
    pub fn new(cache_config: CacheConfig) -> Self {
        let cache = CacheBuilder::new(cache_config.max_capacity)
            .time_to_live(cache_config.time_to_live)
            .build();

        Self {
            profile_req_agent: WebIdProfileReqAgent::new(),
            cache,
        }
    }

    /// Resolve trusted oidc issuers of given webid afresh.
    async fn resolve_fresh(
        &self,
        webid: WebId,
    ) -> Result<Vec<AbsoluteHttpUri>, WebIdTrustedIssuersResolutionError> {
        // Resolve webid profile.
        let profile = self
            .profile_req_agent
            .try_get_profile_document::<HashSet<[SimpleTerm; 3]>>(&webid)
            .inspect_err(|e| {
                error!("Error in resolving web id profile. Error:\n {}", e);
            })
            .await?;

        // > To discover a list of valid issuers, the WebID Profile MUST be
        // > checked for the existence of statements matching
        // > `?webid <http://www.w3.org/ns/solid/terms#oidcIssuer> ?iss .`

        let issuers = profile
            .triples_matching([&webid], [ns::solid::oidcIssuer], Any)
            .filter_map(|tr| {
                tr.unwrap_infallible()
                    .o()
                    .iri()
                    .and_then(|oiri| AbsoluteHttpUri::try_new_from(oiri.as_str()).ok())
            })
            .collect::<Vec<_>>();

        Ok(issuers)
    }

    // TODO resolution from profile req headers as specified in oidc spec.
}
