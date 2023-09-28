//! I define struct to represent `ath` dpop-proof claim.
//!

use std::ops::Deref;

use gdp_rs::Proven;
use picky::hash::HashAlgorithm::SHA2_256;
use serde::{Deserialize, Serialize};

use super::common::base64url_encoded::{Base64UrlEncodingRule, IsValidBase64UrlEncodedValue};

/// A struct representing `ath` dpop-proof claim.
///
/// From spec:
///
/// >  ath: hash of the access token. The value MUST be the result
/// > of a base64url encoding (as defined in Section 2 of RFC7515)
/// > the SHA-256 SHS hash of the ASCII encoding of the
/// > associated access token's value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Ath(Proven<String, IsValidBase64UrlEncodedValue>);

impl From<Ath> for String {
    #[inline]
    fn from(ath: Ath) -> Self {
        ath.0.into_subject()
    }
}

impl Deref for Ath {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl Ath {
    /// Get a new `ath` from given access_token.
    pub fn new(access_token: &str) -> Self {
        Self(
            Proven::void_proven(SHA2_256.digest(access_token.as_bytes()))
                .infer::<Base64UrlEncodingRule<_>>(Default::default()),
        )
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::proof::payload::ath::Ath;

    #[rstest]
    #[case(
        "Kz~8mXK1EalYznwH-LC-1fBAo.4Ljp~zsPE_NeO.gxU",
        "fUHyO2r2Z3DZ53EsNrWBb0xWXoaNy59IiKCAqksmQEo"
    )]
    fn ath_new_works_correctly(#[case] access_token: &str, #[case] expected_hash: &str) {
        assert_eq!(&*Ath::new(access_token), expected_hash);
    }
}
