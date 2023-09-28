//! I define [`Token`] struct corresponding to `token` rule.
//!
use std::{borrow::Borrow, fmt::Display, ops::Deref, str::FromStr};

use ecow::EcoString;
use once_cell::sync::Lazy;
use regex::Regex;

static TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    let tchar_non_alpha_numeric = regex::escape("!#$%&'*+-.^_`|~");
    let token_pattern = format!("^[{}0-9a-zA-Z]+$", tchar_non_alpha_numeric);
    Regex::new(&token_pattern).expect("regex is claimed valid")
});

/// A struct for representing [RFC 9110 token production]
///
/// > Tokens are short textual identifiers that do not include whitespace or delimiters.
/// ```txt
/// token = 1*tchar

/// tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*"
///                 / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
///                 / DIGIT / ALPHA
///                 ; any VCHAR, except delimiters
/// ```
///
/// [RFC 9110 token production]: https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.2
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token(EcoString);

impl Display for Token {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid token source.
#[error("Token source is invalid")]
pub struct InvalidEncodedToken {}

impl FromStr for Token {
    type Err = InvalidEncodedToken;

    #[inline]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if TOKEN_RE.is_match(value) {
            Ok(Self(EcoString::from(value)))
        } else {
            Err(InvalidEncodedToken {})
        }
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Token {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for Token {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Token> for EcoString {
    #[inline]
    fn from(val: Token) -> Self {
        val.0
    }
}

// impl Token {
//     /// Get a new [`Token`] without any checks.
//     ///
//     /// panics if value.len() > 23
//     #[inline]
//     pub(crate) const fn new_small_unchecked(value: &str) -> Self {
//         Self(EcoString::new_inline(value))
//     }
// }

#[cfg(test)]
mod tests {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("a b")]
    #[case("a=b")]
    #[case("a,b")]
    #[case("a;b")]
    #[case("a\"b")]
    fn invalid_tokens_will_be_rejected(#[case] token_str: &str) {
        assert_err_eq!(Token::from_str(token_str), InvalidEncodedToken {});
    }

    #[rstest]
    #[case("abc")]
    #[case("ABC")]
    #[case("rel*")]
    #[case("a.bc")]
    #[case("a_.-+bc")]
    #[case("a#bc")]
    #[case("ab|c1")]
    fn valid_tokens_will_be_parsed(#[case] token_str: &str) {
        assert_ok!(Token::from_str(token_str));
    }
}
