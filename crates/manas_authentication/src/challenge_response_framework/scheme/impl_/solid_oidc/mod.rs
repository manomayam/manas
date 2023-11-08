//! I define an implementation of authentication scheme
//! that confirms to solid-oidc specification.
//!  

use std::{
    borrow::Cow,
    fmt::Debug,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use dpop::{
    http_header::{authorization::credentials::DPoPAuthorizationCredentials, dpop::DPoP},
    proof::{
        context::{DPoPProofContext, KeyBoundAccessToken},
        raw::RawDPoPProof,
        validated::{InvalidDPoPProof, ValidatedDPoPProof},
    },
};
use dyn_problem::{type_::UNKNOWN_IO_ERROR, Problem};
use either::Either;
use futures::future::BoxFuture;
use headers::{authorization::Credentials, Authorization, HeaderMapExt};
use http::{HeaderMap, Method};
use http_headers::{
    common::field::rules::{
        parameter::FieldParameter, parameter_name::FieldParameterName,
        parameter_value::FieldParameterValue, parameters::FieldParameters, token::Token,
    },
    www_authenticate::{Challenge, WWWAuthenticate},
};
use http_uri::{
    invariant::{AbsoluteHttpUri, SecureHttpUri},
    security::transport_policy::{LocalhostExemptingSTP, SecureTransportPolicy},
};
use once_cell::sync::Lazy;
use picky::jose::jwk::JwkSet;
use solid_oidc_types::id_token::{
    context::IdTokenContext,
    raw::RawIdToken,
    validated::{InvalidIdToken, ValidatedIdToken},
};
use tracing::error;
use unicase::Ascii;
use webid::{invariant::SecureWebId, profile_req_agent::ProfileDocResolutionError, WebId};

use self::{
    issuer_jwks::{OidcIssuerJwksResolutionError, OidcIssuerJwksResolver},
    setup::{impl_::BasicSolidOidcDpopSchemeSetup, SolidOidcDpopSchemeSetup},
    trusted_issuers::WebIdTrustedIssuersResolver,
};
use crate::{
    challenge_response_framework::scheme::{
        impl_::solid_oidc::trusted_issuers::WebIdTrustedIssuersResolutionError,
        CRAuthenticationChallenge, CRAuthenticationScheme, CRResolutionResult,
    },
    common::credentials::impl_::basic::{
        BasicAgentCredentials, BasicClientCredentials, BasicIssuerCredentials,
        BasicRequestCredentials,
    },
};

pub mod issuer_jwks;
pub mod setup;
pub mod trusted_issuers;

/// Scheme name static.
static SCHEME_NAME: Lazy<Ascii<Token>> = Lazy::new(|| {
    DPoPAuthorizationCredentials::SCHEME
        .parse()
        .expect("Must be a valid token.")
});

/// Algs field param name for challenge.
static FPN_ALGS: Lazy<FieldParameterName> =
    Lazy::new(|| "algs".parse().expect("Must be a valid name."));

/// Error field param name for challenge.
static FPN_ERROR: Lazy<FieldParameterName> =
    Lazy::new(|| "error".parse().expect("Must be a valid name."));

/// Error description field param name for challenge.
static FPN_ERROR_DESCR: Lazy<FieldParameterName> =
    Lazy::new(|| "error_description".parse().expect("Must be a valid name."));

/// Supported algs as field param value for challenge.
static FPV_SUPPORTED_ALGS: Lazy<FieldParameterValue> = Lazy::new(|| {
    "RS256 RS384 RS512 ES256 ES384 ES512 EdDSA"
        .try_into()
        .expect("Must be a valid value.")
});

/// `invalid_token` field param value for challenge.
/// See: <https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1>
static FPV_INVALID_TOKEN: Lazy<FieldParameterValue> =
    Lazy::new(|| "invalid_token".try_into().expect("Must be a valid value."));

/// `invalid_req` field param value for challenge.
/// See: <https://www.rfc-editor.org/rfc/rfc6750.html#section-3.1>
static _FPV_INVALID_REQ: Lazy<FieldParameterValue> = Lazy::new(|| {
    "invalid_request"
        .try_into()
        .expect("Must be a valid value.")
});

/// `invalid_req` field param value for challenge.
/// See: <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-5-3>
static FPV_INVALID_DPOP_PROOF: Lazy<FieldParameterValue> = Lazy::new(|| {
    "invalid_dpop_proof"
        .try_into()
        .expect("Must be a valid value.")
});

/// An implementation of authentication scheme
/// that confirms to solid-oidc specification.
#[derive(Debug)]
pub struct SolidOidcDpopScheme<Setup: SolidOidcDpopSchemeSetup> {
    /// Webid trusted issuers resolver.
    pub webid_issuers_resolver: Arc<Setup::WebIdIssuersResolver>,

    /// Oidc issuer jwks resolver.
    pub issuer_jwks_resolver: Arc<Setup::IssuerJwksResolver>,

    /// Dpop verification time leeway
    pub dpop_time_leeway: Duration,
    // TODO jti, nonce cache for jti verification.
}

impl<Setup: SolidOidcDpopSchemeSetup> Clone for SolidOidcDpopScheme<Setup> {
    fn clone(&self) -> Self {
        Self {
            webid_issuers_resolver: self.webid_issuers_resolver.clone(),
            issuer_jwks_resolver: self.issuer_jwks_resolver.clone(),
            dpop_time_leeway: self.dpop_time_leeway,
        }
    }
}

impl<Setup: SolidOidcDpopSchemeSetup> SolidOidcDpopScheme<Setup> {
    /// Resolve credentials for given authentication headers.
    #[tracing::instrument(
        skip_all,
        name = "SolidOidcDpopScheme::resolve_credentials",
        fields(uri, method, h_authorization, h_dpop,)
    )]
    async fn resolve_credentials(
        self,
        uri: AbsoluteHttpUri,
        method: Method,
        h_authorization: Authorization<DPoPAuthorizationCredentials>,
        h_dpop: DPoP,
    ) -> CRResolutionResult<BasicRequestCredentials> {
        // Deserialize id token jwt.
        let raw_id_token = RawIdToken::decode(Cow::Owned(h_authorization.0.access_token.into()))
            .map_err(|e| {
                error!("Error in deserializing id token jwt. Error:\n {}", e);
                Self::challenge(
                    Some(&*FPV_INVALID_TOKEN),
                    Some("Id token is not structurally valid."),
                )
            })?;

        let id_token_claims = &raw_id_token.decoded_essence().claims;
        let iss = id_token_claims.iss.clone();
        let webid = id_token_claims.webid.clone();
        let azp = id_token_claims.azp.clone();

        // Verify security as per policy.
        self.verify_stp_security(&webid, &iss)?;

        // Verify that issuer is trusted.
        self.verify_issuer_trusted(&webid, &iss).await?;

        // Validate id token structurally.
        let verified_id_token = self.verify_id_token(raw_id_token).await?;

        // Verify dpop-proof.
        let _verified_dpop_proof =
            self.verify_dpop_proof(uri, method, h_dpop.0, verified_id_token)?;

        Ok(BasicRequestCredentials {
            of_agent: Some(BasicAgentCredentials { webid }),
            of_client: Some(BasicClientCredentials {
                client_id: azp,
                client_web_id: None,
            }),
            of_issuer: Some(BasicIssuerCredentials { uri: iss }),
        })
    }

    /// Verify uri security asper stp.
    fn verify_stp_security(&self, webid: &WebId, iss: &AbsoluteHttpUri) -> CRResolutionResult<()> {
        // Verify webid security as per stp.
        let _webid_secure = SecureWebId::<Setup::SecureTransportPolicy>::try_new(webid.clone())
            .map_err(|_| {
                Self::challenge(Some(&*FPV_INVALID_TOKEN), Some("Webid uri is insecure."))
            })?;

        // Verify issuer uri security as per stp.
        let _iss_secure = SecureHttpUri::<Setup::SecureTransportPolicy>::try_new(
            iss.as_ref().clone(),
        )
        .map_err(|_| Self::challenge(Some(&*FPV_INVALID_TOKEN), Some("Issuer uri is insecure.")))?;

        Ok(())
    }

    /// Verify if issuer is trusted by webid.
    async fn verify_issuer_trusted(
        &self,
        webid: &WebId,
        iss: &AbsoluteHttpUri,
    ) -> CRResolutionResult<()> {
        // Resolve trusted issuers.
        let trusted_issuers = self
            .webid_issuers_resolver
            .resolve(webid.clone())
            .await
            .map_err(|e| {
                // Resolve error description.
                match e.as_ref() {
                    WebIdTrustedIssuersResolutionError::ProfileDocResolutionError(ie) => match ie {
                        ProfileDocResolutionError::InvalidDerefResponse => {
                            error!("Invalid webid profile deref response.");
                            Self::challenge(
                                Some(&*FPV_INVALID_TOKEN),
                                Some("Invalid webid profile deref response."),
                            )
                        }
                        ProfileDocResolutionError::InvalidProfileContent => {
                            error!("Invalid webid profile content.");
                            Self::challenge(
                                Some(&*FPV_INVALID_TOKEN),
                                Some("Invalid webid profile content."),
                            )
                        }
                        ProfileDocResolutionError::UnknownIoError(_) => {
                            error!("Unknown io error in resolving webid profile.");
                            Either::Right(UNKNOWN_IO_ERROR.new_problem())
                        }
                    },
                }
            })?;

        // Ensure issuer is trusted.
        if !trusted_issuers.contains(iss) {
            return Err(Self::challenge(
                Some(&*FPV_INVALID_TOKEN),
                Some("Token issuer is not configured as trusted by webid."),
            ));
        }

        Ok(())
    }

    /// Resolve issuer jwks.
    async fn resolve_issuer_jwks(&self, iss: &AbsoluteHttpUri) -> CRResolutionResult<JwkSet> {
        self.issuer_jwks_resolver
            .resolve(iss.clone())
            .await
            .map_err(|e| {
                error!("Error in retrieving issuer jwks.");
                match e.as_ref() {
                    OidcIssuerJwksResolutionError::UnknownIoError(_) => {
                        Either::Right(UNKNOWN_IO_ERROR.new_problem())
                    }
                    OidcIssuerJwksResolutionError::InvalidOidcIssuerConfigResponse => {
                        Self::challenge(
                            Some(&*FPV_INVALID_TOKEN),
                            Some("Invalid issuer oidc config response."),
                        )
                    }
                    OidcIssuerJwksResolutionError::InvalidJwksDerefResponse => Self::challenge(
                        Some(&*FPV_INVALID_TOKEN),
                        Some("Invalid issuer  jwks_uri deref response."),
                    ),
                }
            })
    }

    /// Verify the id token.
    async fn verify_id_token(
        &self,
        raw_id_token: RawIdToken<'static>,
    ) -> CRResolutionResult<ValidatedIdToken> {
        // Resolve issuer jwks.
        let issuer_jwks = self
            .resolve_issuer_jwks(&raw_id_token.decoded_essence().claims.iss)
            .await?;

        // Verify id token.
        ValidatedIdToken::try_new(
            raw_id_token,
            IdTokenContext {
                current_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Must be valid.")
                    .as_secs(),
                issuer_jwks,
            },
        )
        .map_err(|e| {
            error!("Error in validating access token.");
            let error_descr = match e.error {
                InvalidIdToken::InvalidAudClaim => "Invalid aud claim.",
                InvalidIdToken::IsExpired => "Id token is expired.",
                InvalidIdToken::InvalidIatClaim => "Invalid iat claim.",
                InvalidIdToken::UnsupportedAlg => "Unsupported alg.",
                InvalidIdToken::UnresolvedIssuerPublicKey => {
                    "Issuer public key unresolved from it's config."
                }
                InvalidIdToken::InvalidIssuerPublicKeyJwk(_) => "Issuer public key jwk is invalid.",
                InvalidIdToken::InvalidSignature(_) => "Invalid signature.",
            };
            Self::challenge(Some(&*FPV_INVALID_TOKEN), Some(error_descr))
        })
    }

    /// Verify dpop-proof.
    fn verify_dpop_proof(
        &self,
        uri: AbsoluteHttpUri,
        method: Method,
        raw_dpop_proof: RawDPoPProof<'static>,
        id_token: ValidatedIdToken,
    ) -> CRResolutionResult<ValidatedDPoPProof> {
        // Construct dpop-proof context.
        let context = DPoPProofContext {
            req_method: method,
            req_uri: uri.into_subject(),
            req_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Must be valid.")
                .as_secs()
                .try_into()
                .expect("Must be representable."),
            active_nonce: None,
            nonce_timestamp: None,
            time_leeway: self.dpop_time_leeway.as_secs().try_into().unwrap_or(120),
            key_bound_access_token: Some(KeyBoundAccessToken {
                access_token: id_token.compact_repr().to_string(),
                bound_key_jkt: id_token.decoded_essence().claims.cnf.jkt.clone(),
            }),
        };

        ValidatedDPoPProof::try_new(raw_dpop_proof, context).map_err(|e| {
            error!("Error in dpop proof validation.");
            let err_descr = match e.error {
                InvalidDPoPProof::InvalidPublicKeyJwk(_) => {
                    "Invalid jwk claim in dpop-proof header."
                }
                InvalidDPoPProof::InvalidSignature(_) => "Invalid signature.",
                InvalidDPoPProof::HtmClaimMismatch => "htm claim mismatch.",
                InvalidDPoPProof::HtuClaimMismatch => "htu claim mismatch.",
                // TODO Should provide proper support.
                InvalidDPoPProof::NonceClaimMismatch => "nonce claim mismatch",
                InvalidDPoPProof::AthClaimMismatch => "ath claim mismatch.",
                InvalidDPoPProof::BindingKeyMisMatch => "Binding key mismatch.",
                InvalidDPoPProof::TimestampOutOfWindow => "Timestamp out of window.",
            };
            Self::challenge(Some(&*FPV_INVALID_DPOP_PROOF), Some(err_descr))
        })
    }

    /// Return a challenge  with given params.
    /// @see: <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-7.1-9>.
    fn challenge(
        error: Option<&FieldParameterValue>,
        error_descr: Option<&str>,
    ) -> Either<CRAuthenticationChallenge, Problem> {
        // Challenge's ext-params.
        let mut ext_params = vec![
            // > An algs parameter SHOULD be included to signal to
            // > the client the JWS algorithms that are acceptable for the DPoP proof JWT.
            FieldParameter {
                name: (*FPN_ALGS).clone(),
                value: (*FPV_SUPPORTED_ALGS).clone(),
            },
        ];

        // > An error parameter ([RFC6750], Section 3) SHOULD be included to
        // > indicate the reason why the request was declined, if the
        // > request included an access token but failed authentication.
        if let Some(error) = error {
            ext_params.push(FieldParameter {
                name: (*FPN_ERROR).clone(),
                value: error.clone(),
            });
        }

        // > An error_description parameter ([RFC6750], Section 3) MAY be
        // > included along with the error parameter to provide
        // > developers a human-readable explanation that is not meant to be displayed to end-users.
        if let Some(error_descr) = error_descr.and_then(|v| FieldParameterValue::try_from(v).ok()) {
            ext_params.push(FieldParameter {
                name: (*FPN_ERROR_DESCR).clone(),
                value: error_descr,
            });
        }

        Either::Left(CRAuthenticationChallenge {
            www_authenticate: WWWAuthenticate {
                challenges: vec![Challenge {
                    auth_scheme: (*SCHEME_NAME).clone(),
                    ext_info: Either::Right(FieldParameters::new(ext_params)),
                }],
            },
            ext_headers: Default::default(),
        })
    }
}

impl<Setup: SolidOidcDpopSchemeSetup> CRAuthenticationScheme for SolidOidcDpopScheme<Setup> {
    type Credentials = BasicRequestCredentials;

    fn resolve_or_challenge(
        &self,
        uri: &AbsoluteHttpUri,
        method: &Method,
        headers: &HeaderMap,
    ) -> BoxFuture<'static, CRResolutionResult<BasicRequestCredentials>> {
        let rh_authorization = headers
            .typed_get::<Authorization<DPoPAuthorizationCredentials>>()
            .ok_or_else(|| {
                error!("No Authorization header for this scheme.");
                Self::challenge(None, None)
            });

        let rh_dpop = headers.typed_get::<DPoP>().ok_or_else(|| {
            error!("No DPoP header.");
            Self::challenge(Some(&*FPV_INVALID_DPOP_PROOF), None)
        });

        let this = self.clone();
        let uri = uri.clone();
        let method = method.clone();

        Box::pin(async move {
            this.resolve_credentials(uri, method, rh_authorization?, rh_dpop?)
                .await
                // Convert any io problems to re-challenges.
                .map_err(|e| match e {
                    Either::Left(challenge) => Either::Left(challenge),
                    Either::Right(problem) => {
                        if UNKNOWN_IO_ERROR.is_type_of(&problem) {
                            Self::challenge(Some(&*FPV_INVALID_TOKEN), Some("Unknown io error."))
                        } else {
                            Either::Right(problem)
                        }
                    }
                })
        })
    }
}

/// A struct for representing cache config.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum capacity of the cache.
    pub max_capacity: u64,

    /// Each entity's time to live.
    pub time_to_live: Duration,
}

/// Type of default solid oidc dpop scheme.
pub type DefaultSolidOidcDpopScheme =
    SolidOidcDpopScheme<BasicSolidOidcDpopSchemeSetup<LocalhostExemptingSTP>>;

impl<STP: SecureTransportPolicy> Default
    for SolidOidcDpopScheme<BasicSolidOidcDpopSchemeSetup<STP>>
{
    fn default() -> Self {
        Self {
            webid_issuers_resolver: Default::default(),
            issuer_jwks_resolver: Default::default(),
            dpop_time_leeway: Duration::from_secs(120),
        }
    }
}
