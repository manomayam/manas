//! I define an implementation of [`CRAuthenticationScheme`] that routes to inner schemes.

use std::{collections::HashMap, str::FromStr, sync::Arc};

use either::Either;
use futures::future::BoxFuture;
use http::{header::AUTHORIZATION, HeaderMap, Method};
use http_headers::{common::field::rules::token::Token, www_authenticate::WWWAuthenticate};
use http_uri::invariant::AbsoluteHttpUri;
use itertools::Itertools;
use tracing::warn;

use crate::{
    challenge_response_framework::scheme::{
        CRAuthenticationChallenge, CRAuthenticationScheme, CRResolutionResult,
        DynCRAuthenticationScheme,
    },
    common::credentials::RequestCredentials,
};

/// An implementation of [`CRAuthenticationScheme`] that routes to inner schemes.
#[derive(Debug, Clone)]
pub struct UnionCRAuthenticationScheme<C: RequestCredentials> {
    /// Scheme map.
    pub schemes: HashMap<Token, Arc<DynCRAuthenticationScheme<C>>>,

    /// Default challenge schemes.
    /// Their challenges will be included in effective challenge
    /// when no credentials are supplied in headers.
    pub default_challenge_schemes: Vec<Token>,
}

impl<C: RequestCredentials> CRAuthenticationScheme for UnionCRAuthenticationScheme<C> {
    type Credentials = C;

    fn resolve_or_challenge(
        &self,
        uri: &AbsoluteHttpUri,
        method: &Method,
        headers: &HeaderMap,
    ) -> BoxFuture<'static, CRResolutionResult<Self::Credentials>> {
        // Try resolve authn scheme name.
        let scheme_name = headers
            .get_all(AUTHORIZATION)
            .iter()
            // Multiple authn schemes will be rejected.
            .exactly_one()
            .ok()
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Token::from_str(v).ok());

        // Check if any inner scheme matches the request.
        if let Some(scheme) = scheme_name
            .as_ref()
            .and_then(|n| self.schemes.get(n))
            .cloned()
        {
            // Delegate to matched scheme.
            return scheme.resolve_or_challenge(uri, method, headers);
        } else {
            // Return the default challenges.
            let schemes = self
                .default_challenge_schemes
                .iter()
                .filter_map(|n| self.schemes.get(n).cloned().map(|s| (n.clone(), s.clone())))
                .collect::<Vec<_>>();

            let uri = uri.clone();
            let method = method.clone();
            let dummy_headers = HeaderMap::new();

            Box::pin(async move {
                let mut wwwauthn_challenges = vec![];
                // Collect challenges from inner schemes.
                for (scheme_name, scheme) in schemes {
                    if let Err(e) = scheme
                        .resolve_or_challenge(&uri, &method, &dummy_headers)
                        .await
                    {
                        match e {
                            // On challenge.
                            Either::Left(challenge) => {
                                wwwauthn_challenges.extend(challenge.www_authenticate.challenges)
                            }
                            // On unknown problem, skip.
                            Either::Right(_ie) => {
                                warn!(
                                    "CRAuthentication scheme {} raises unknown error on no creds.",
                                    scheme_name
                                );
                            }
                        }
                    }
                }

                Err(Either::Left(CRAuthenticationChallenge {
                    www_authenticate: WWWAuthenticate {
                        challenges: wwwauthn_challenges,
                    },
                    ext_headers: Default::default(),
                }))
            })
        }
    }
}
