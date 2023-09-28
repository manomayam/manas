//! I define types to represent payload claims in
//! solid-oidc-id-token .

use std::{collections::HashMap, error::Error, fmt};

use dpop::proof::payload::jkt::Jkt;
use http_uri::invariant::AbsoluteHttpUri;
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use webid::WebId;

/// A struct for representing oidc id token payload claims as required by solid-oidc.
/// See <https://solidproject.org/TR/oidc#tokens-id>
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdTokenClaims {
    /// Issuer.
    pub iss: AbsoluteHttpUri,

    /// Token subject.
    pub sub: String,

    /// Authorized party.
    #[serde(alias = "client_id")]
    pub azp: String,

    /// Audience.
    // NOTE: for NSS idp compat, allows deserializing from a string.
    // TODO remove special handling.
    #[serde(deserialize_with = "string_or_vec")]
    pub aud: Vec<String>,

    /// Expiration unix timestamp in seconds.
    pub exp: u64,

    /// Issued at unix timestamp in seconds.
    pub iat: u64,

    /// Subject's webid.
    pub webid: WebId,

    /// Dpop cnf claim.
    pub cnf: DPoPCnfClaim,

    /// Any additional claims.
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Struct to represent `cnf` claim in header of
/// dpop bound access_tokens.
/// See <https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-6.1>
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DPoPCnfClaim {
    /// Json key thumbprint of the the DPoP public key (in JWK format)
    /// to which the access token is bound.
    pub jkt: Jkt,
}

mod predicate {
    use gdp_rs::predicate::Predicate;

    use super::IdTokenClaims;

    /// A predicate over [`SolidOidcIdTokenClaims`] that
    /// asserts for their validity.
    #[derive(Debug)]
    pub struct AreValidSolidOidcIdTokenClaims;

    impl Predicate<IdTokenClaims> for AreValidSolidOidcIdTokenClaims {
        fn label() -> std::borrow::Cow<'static, str> {
            "AreValidSolidOidcIdTokenClaims".into()
        }
    }
}

// Adapted from https://serde.rs/string-or-struct.html
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or seq")
        }

        fn visit_str<E>(self, value: &str) -> Result<Vec<String>, E>
        where
            E: Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}
