//!

use std::ops::Deref;

use gdp_rs::Proven;
use picky::{hash::HashAlgorithm::SHA2_256, jose::jwk::Jwk};
use serde::{Deserialize, Serialize};

use super::common::base64url_encoded::{Base64UrlEncodingRule, IsValidBase64UrlEncodedValue};

/// A struct for representing ``jkt` values in different dpop-proof claims..
///
/// From spec:
///
/// >  jkt: The value of the jkt member MUST be the base64url encoding (as defined in RFC7515)
/// of the JWK SHA-256 Thumbprint (according to RFC7638) of the public key (in JWK format)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Jkt(Proven<String, IsValidBase64UrlEncodedValue>);

impl From<Jkt> for String {
    #[inline]
    fn from(ath: Jkt) -> Self {
        ath.0.into_subject()
    }
}

impl Deref for Jkt {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl Jkt {
    /// Get a new `jkt` from given jwk.
    pub fn new(jwk: &Jwk) -> Self {
        // Calculate canonical json of the jwk as per rfc7638.
        // TODO must address case of non-rsa keys once picky supports them.
        let canonical_jwk =
            canonical_json::to_string(&serde_json::to_value(&jwk.key).expect("Must succeed."))
                .expect("Must be successful.");

        Self(
            Proven::void_proven(SHA2_256.digest(canonical_jwk.as_bytes()))
                .infer::<Base64UrlEncodingRule<_>>(Default::default()),
        )
    }
}

#[cfg(test)]
mod tests {
    use picky::jose::jwk::Jwk;
    use rstest::*;

    use super::Jkt;

    #[rstest]
    #[case(
        r#"
            {
                "kty": "RSA",
                "n": "0vx7agoebGcQSuuPiLJXZptN9nndrQmbXEps2aiAFbWhM78LhWx4cbbfAAtVT86zwu1RK7aPFFxuhDR1L6tSoc_BJECPebWKRXjBZCiFV4n3oknjhMstn64tZ_2W-5JsGY4Hc5n9yBXArwl93lqt7_RN5w6Cf0h4QyQ5v-65YGjQR0_FDW2QvzqY368QQMicAtaSqzs8KJZgnYb9c7d0zgdAZHzu6qMQvRL5hajrn1n91CbOpbISD08qNLyrdkt-bFTWhAI4vMQFh6WeZu0fM4lFd2NcRwr3XPksINHaQ-G_xBniIqbw0Ls1jF44-csFCur-kEgU8awapJzKnqDKgw",
                "e": "AQAB",
                "alg": "RS256",
                "kid": "2011-04-29"
            }
        "#,
        "NzbLsXh8uDCcd-6MNwXF4W_7noWXFZAfHkxZsRGC9Xs"
    )]
    fn new_from_jwk_works_correctly(#[case] jwk_json: &str, #[case] expected_jkt: &str) {
        let jwk = serde_json::from_str::<Jwk>(jwk_json).expect("Claimed valid json");
        let jkt = Jkt::new(&jwk);
        assert_eq!(&*jkt, expected_jkt);
    }
}
