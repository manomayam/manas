//! I define type for representing `token68` production as defined in rfc9110.
//!

use std::{ops::Deref, str::FromStr};

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex to match a `token68` value.
static TOKEN68_RE: Lazy<Regex> = Lazy::new(|| {
    let t68char_non_alpha_numeric = regex::escape("-._~+/");
    Regex::new(&(format!("^[{}0-9a-zA-Z]+=*$", t68char_non_alpha_numeric)))
        .expect("regex is claimed valid")
});

/// A type for representing `token68` production.
///
/// ```txt
/// token68        = 1*( ALPHA / DIGIT /
///                        "-" / "." / "_" / "~" / "+" / "/" ) *"="
/// ```
///
///  The token68 syntax allows the 66 unreserved URI characters,
/// plus a few others, so that it can hold a base64, base64url (URL and filename safe alphabet),
/// base32, or base16 (hex) encoding, with or without padding,
/// but excluding whitespace (RFC4648).
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token68(String);

impl TryFrom<String> for Token68 {
    type Error = InvalidToken68;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if TOKEN68_RE.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidToken68(value))
        }
    }
}

impl FromStr for Token68 {
    type Err = InvalidToken68;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl From<Token68> for String {
    #[inline]
    fn from(token: Token68) -> Self {
        token.0
    }
}

impl Deref for Token68 {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Token68 {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

/// An error representing invalid token68.
#[derive(Debug, Clone, thiserror::Error)]
#[error("\"{0}\" is not a valid token68.")]
pub struct InvalidToken68(String);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::Token68;

    #[rstest]
    #[case::empty("", false)]
    #[case::ws_only("  ", false)]
    #[case::unicode("రామాయణం", false)]
    #[case::disallowed_ascii("abc,def", false)]
    #[case::disallowed_ascii("abc,def", false)]
    #[case::disallowed_ascii("abc,def", false)]
    #[case("cK76", true)]
    #[case("Kz~8mXK1EalYznwH-LC-1fBAo.4Ljp~zsPE_NeO.gxU", true)]
    #[case("Kz~8mXK1EalYznwH-LC-1fBAo.4Ljp~zsPE_NeO.gxU==", true)]
    fn token68_parsing_works_correctly(#[case] value: &str, #[case] expected_is_valid: bool) {
        assert_eq!(Token68::from_str(value).is_ok(), expected_is_valid);
    }
}
