//! I define rust model for `ALPHA` abnf production rule.
//!
use std::{borrow::Borrow, fmt::Display, ops::Deref, str::FromStr};

use ecow::EcoString;
use once_cell::sync::Lazy;
use regex::Regex;

static ALPHA_RE: Lazy<Regex> = Lazy::new(|| {
    let alpha_pattern = "^[a-zA-Z]+$";
    Regex::new(alpha_pattern).expect("regex is claimed valid")
});

/// A struct for representing `ALPHA`  abnf production.
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Alpha(EcoString);

impl Display for Alpha {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Invalid encoded alpha production.
#[error("Invalid encoded alpha production.")]
pub struct InvalidEncodedAlpha {}

impl FromStr for Alpha {
    type Err = InvalidEncodedAlpha;

    #[inline]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if ALPHA_RE.is_match(value) {
            Ok(Self(EcoString::from(value)))
        } else {
            Err(InvalidEncodedAlpha {})
        }
    }
}

impl AsRef<str> for Alpha {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Alpha {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for Alpha {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Alpha {
    /// Get a new [`Alpha`] without any checks.
    ///
    /// panics if value.len() > 15
    #[inline]
    pub(crate) const fn new_small_unchecked(value: &str) -> Self {
        Self(EcoString::inline(value))
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("a b")]
    #[case("a=b")]
    #[case("a,b")]
    #[case("a_b")]
    #[case("a1")]
    #[case("a\"b")]
    fn invalid_str_will_be_rejected(#[case] token_str: &str) {
        assert_err_eq!(Alpha::from_str(token_str), InvalidEncodedAlpha {});
    }

    #[rstest]
    #[case("abc")]
    #[case("ABC")]
    #[case("ABa")]
    fn valid_str_will_be_parsed(#[case] token_str: &str) {
        assert_ok!(Alpha::from_str(token_str));
    }
}
