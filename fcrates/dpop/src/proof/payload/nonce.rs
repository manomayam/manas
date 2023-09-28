//! I define struct to represent `nonce` dpop-proof claim.
//!

use std::{ops::Deref, str::FromStr};

use gdp_rs::{proven::ProvenError, Proven};
use serde::{Deserialize, Serialize};

use self::predicate::{InvalidNonce, IsValidNonce};

/// A struct representing `nonce` dpop-proof claim.
///
/// From spec:
///
/// >  nonce: A recent nonce provided via the DPoP-Nonce HTTP header.
///
/// [Syntax](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-8.1):
///
/// ```txt
/// The nonce syntax in ABNF as used by [RFC6749] (which is the same as the scope-token syntax) is:
///
///  nonce = 1*NQCHAR
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nonce(Proven<String, IsValidNonce>);

impl TryFrom<String> for Nonce {
    type Error = ProvenError<String, InvalidNonce>;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(Proven::try_new(value)?))
    }
}

impl FromStr for Nonce {
    type Err = ProvenError<String, InvalidNonce>;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl From<Nonce> for String {
    #[inline]
    fn from(val: Nonce) -> Self {
        val.0.into_subject()
    }
}

impl Deref for Nonce {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

mod predicate {
    use std::borrow::Borrow;

    use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};
    use once_cell::sync::Lazy;
    use regex::Regex;

    /// Regex to match a dpop nonce value.
    // #[allow(clippy::needless_raw_string_hashes)]
    static NONCE_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"^[\x21\x23-\x5b\x5d-\x7e]+$"#).expect("Must be valid regex."));

    /// A predicate over `&str`, asserting that it is a valid dpop nonce value.
    #[derive(Debug)]
    pub struct IsValidNonce;

    impl<T: Borrow<str>> Predicate<T> for IsValidNonce {
        fn label() -> std::borrow::Cow<'static, str> {
            "IsValidNonce".into()
        }
    }

    /// An error type for invalid base64url values.
    #[derive(Debug, thiserror::Error)]
    #[error("Invalid nonce value.")]
    pub struct InvalidNonce;

    impl<T: Borrow<str>> SyncEvaluablePredicate<T> for IsValidNonce {
        type EvalError = InvalidNonce;

        fn evaluate_for(sub: &T) -> Result<(), Self::EvalError> {
            if NONCE_RE.is_match(sub.borrow()) {
                Ok(())
            } else {
                Err(InvalidNonce)
            }
        }
    }

    impl<T: Borrow<str>> PurePredicate<T> for IsValidNonce {}
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::*;

    use crate::proof::payload::nonce::Nonce;

    #[rstest]
    #[case("", false)]
    #[case("abc def", false)]
    #[case("ey\"J7S_zG.eyJbYu3.xQmBj-1", false)]
    #[case("abc~def", true)]
    #[case("a=b", true)]
    #[case("eyJ7S_zG.eyJbYu3.xQmBj-1", true)]
    fn nonce_parse_works_correctly(#[case] input_str: &str, #[case] expected_is_valid: bool) {
        assert_eq!(Nonce::from_str(input_str).is_ok(), expected_is_valid);
    }
}
