//! This module defines types to represent http message
//! components and their properties.
//!

use core::str::FromStr;
use std::ops::Deref;

use ecow::EcoString;
use once_cell::sync::Lazy;
use regex_lite::Regex;
use sfv::{BareItem, Parameters};

static FIELD_NAME_LOWER_RE: Lazy<Regex> = Lazy::new(|| {
    let tchar_non_alpha_numeric = regex_lite::escape("!#$%&'*+-.^_`|~");
    let token_pattern = format!("^[{}0-9a-z]+$", tchar_non_alpha_numeric);
    Regex::new(&token_pattern).expect("regex is claimed valid")
});

/// RFC 8941 param-key matcher
///
/// > param-key     = key
/// > key           = ( lcalpha / "*" )
/// >                 *( lcalpha / DIGIT / "_" / "-" / "." / "*" )
/// > lcalpha       = %x61-7A ; a-z
static SFV_KEY_RE: Lazy<Regex> = Lazy::new(|| {
    let key_char_non_alpha_numeric = regex_lite::escape("_-.*");
    let key_pattern = format!("^[a-z\\*][{}0-9a-z]+$", key_char_non_alpha_numeric);
    Regex::new(&key_pattern).expect("regex is claimed valid")
});

/// Field component param `sf`.
pub static COMP_PARAM_SF: SfvKey = SfvKey(EcoString::inline("sf"));

/// Field component param `key`.
pub static COMP_PARAM_KEY: SfvKey = SfvKey(EcoString::inline("key"));

/// Field component param `bs`.
pub static COMP_PARAM_BS: SfvKey = SfvKey(EcoString::inline("bs"));

/// Field component param `tr`.
pub static COMP_PARAM_TR: SfvKey = SfvKey(EcoString::inline("tr"));

/// Field component param `req`.
pub static COMP_PARAM_REQ: SfvKey = SfvKey(EcoString::inline("req"));

/// Http field component name.
///
/// [Spec](https://datatracker.ietf.org/doc/html/draft-ietf-httpbis-message-signatures#section-2.1):
///
/// > The component name for an HTTP field is the lowercased
/// form of its field name as defined in Section 5.1 of
/// [HTTP]. While HTTP field names are case-insensitive,
/// implementations MUST use lowercased field names (e.g.,
/// content-type, date, etag) when using them as component
/// names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldComponentName(EcoString);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Invalid field component name.")]
pub struct InvalidFieldComponentName;

impl FromStr for FieldComponentName {
    type Err = InvalidFieldComponentName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if FIELD_NAME_LOWER_RE.is_match(s) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidFieldComponentName)
        }
    }
}

impl Deref for FieldComponentName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for FieldComponentName {
    #[inline]
    fn as_ref(&self) -> &str {
        &*self
    }
}

/// Derived component name.
///
/// > Derived component names MUST start with the "at" @
/// character. This differentiates derived component names
/// from HTTP field names
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DerivedComponentName(EcoString);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Invalid derived component name.")]
pub struct InvalidDerivedComponentName;

impl FromStr for DerivedComponentName {
    type Err = InvalidDerivedComponentName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('@') && FIELD_NAME_LOWER_RE.is_match(&s[1..]) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidDerivedComponentName)
        }
    }
}

impl Deref for DerivedComponentName {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl AsRef<str> for DerivedComponentName {
    #[inline]
    fn as_ref(&self) -> &str {
        &*self.0
    }
}

/// Each component name is either an HTTP field name
/// (Section 2.1) or a registered derived component name (Section 2.2)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentName {
    /// Http field name.
    Field(FieldComponentName),

    /// Derived component name.
    Derived(DerivedComponentName),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Invalid component name.")]
pub struct InvalidComponentName;

impl FromStr for ComponentName {
    type Err = InvalidComponentName;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = FieldComponentName::from_str(s) {
            Ok(Self::Field(n))
        } else if let Ok(n) = DerivedComponentName::from_str(s) {
            Ok(Self::Derived(n))
        } else {
            Err(InvalidComponentName)
        }
    }
}

impl Deref for ComponentName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            ComponentName::Field(n) => &*n,
            ComponentName::Derived(n) => &*n,
        }
    }
}

impl AsRef<str> for ComponentName {
    #[inline]
    fn as_ref(&self) -> &str {
        &*self
    }
}

/// Structured filed value key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SfvKey(EcoString);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Invalid derived component name.")]
pub struct InvalidSfvKey;

impl FromStr for SfvKey {
    type Err = InvalidSfvKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if SFV_KEY_RE.is_match(&s[1..]) {
            Ok(Self(s.into()))
        } else {
            Err(InvalidSfvKey)
        }
    }
}

impl Deref for SfvKey {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl AsRef<str> for SfvKey {
    #[inline]
    fn as_ref(&self) -> &str {
        &*self.0
    }
}

/// A component identifier is composed of a component name and
/// any parameters associated with that name.
#[derive(Debug, Clone)]
pub struct ComponentId {
    /// Component name.
    pub name: ComponentName,

    /// Component params.
    params: Parameters,
}

impl PartialEq for ComponentId {
    fn eq(&self, other: &Self) -> bool {
        (self.name == other.name) && (self.params.len() == other.params.len()) && (self.params.iter().for_each(|(k, v)| self.params.g))
    }
}

impl ComponentId {
    /// Get a new component id with given flag set to true.
    pub fn with_flag(mut self, param_key: SfvKey) -> Self {
        self.params
            .insert(param_key.as_ref().to_owned(), BareItem::Boolean(true));
        self
    }

    /// With component id param `sf`.
    #[inline]
    pub fn with_flag_sf(self) -> Self {
        self.with_flag(COMP_PARAM_SF.clone())
    }

    /// With component id param `req`.
    #[inline]
    pub fn with_flag_req(self) -> Self {
        self.with_flag(COMP_PARAM_REQ.clone())
    }

    /// With component id param `bs`.
    #[inline]
    pub fn with_flag_bs(self) -> Self {
        self.with_flag(COMP_PARAM_BS.clone())
    }

    /// With component id param `tr`.
    #[inline]
    pub fn with_flag_tr(self) -> Self {
        self.with_flag(COMP_PARAM_TR.clone())
    }

    pub fn with_param_key(mut self, key_name: SfvKey) -> Self {
        self.params.insert(
            COMP_PARAM_KEY.as_ref().to_owned(),
            BareItem::String(key_name.as_ref().to_owned()),
        );
        self
    }

    /// Get params of the component id.
    #[inline]
    pub fn params(&self) -> &Parameters {
        &self.params
    }
}

#[derive(Debug, Clone)]
pub struct CoveredComponentId(pub ComponentId);

/// From spec:
///
/// > The order of parameters MUST be preserved when
/// processing a component identifier (such as when parsing
/// during verification), but the order of parameters is not
/// significant when comparing two component identifiers for
/// equality checks.
impl PartialEq for CoveredComponentId {
    fn eq(&self, other: &Self) -> bool {
        (self.0.name == other.0.name)
            && (self.0.params.len() == other.0.params.len())
            && (self
                .0
                .params
                .iter()
                .all(|(k, v)| other.0.params.get(k) == Some(v)))
    }
}

#[cfg(test)]
mod tests_field_comp_name {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("a b")]
    #[case("a=b")]
    #[case("a,b")]
    #[case("a;b")]
    #[case("a\"b")]
    #[case("ABC")]
    #[case("Content-Type")]
    #[case("Link")]
    #[case("@method")]
    fn invalid_field_comp_name_will_be_rejected(#[case] token_str: &str) {
        assert_err_eq!(
            FieldComponentName::from_str(token_str),
            InvalidFieldComponentName {}
        );
    }

    #[rstest]
    #[case("abc")]
    #[case("rel*")]
    #[case("a.bc")]
    #[case("a_.-+bc")]
    #[case("a#bc")]
    #[case("ab|c1")]
    #[case("content-type")]
    #[case("link")]
    fn valid_field_comp_name_will_be_parsed(#[case] token_str: &str) {
        assert_ok!(FieldComponentName::from_str(token_str));
    }
}

#[cfg(test)]
mod tests_derived_comp_name {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("@a b")]
    #[case("@a=b")]
    #[case("@a,b")]
    #[case("@a;b")]
    #[case("@a\"b")]
    #[case("@ABC")]
    #[case("@Content-Type")]
    #[case("@Link")]
    #[case("method")]
    #[case("request-target")]
    fn invalid_derived_comp_name_will_be_rejected(#[case] token_str: &str) {
        assert_err_eq!(
            DerivedComponentName::from_str(token_str),
            InvalidDerivedComponentName {}
        );
    }

    #[rstest]
    #[case("@abc")]
    #[case("@rel*")]
    #[case("@a.bc")]
    #[case("@a_.-+bc")]
    #[case("@a#bc")]
    #[case("@ab|c1")]
    #[case("@method")]
    #[case("@request-target")]
    fn valid_derived_comp_name_will_be_parsed(#[case] token_str: &str) {
        assert_ok!(DerivedComponentName::from_str(token_str));
    }
}

#[cfg(test)]
mod tests_component_name {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("a b")]
    #[case("a=b")]
    #[case("a,b")]
    #[case("a;b")]
    #[case("a\"b")]
    #[case("ABC")]
    #[case("@method=")]
    #[case("Content type")]
    #[case("Content-type")]
    fn invalid_comp_name_will_be_rejected(#[case] token_str: &str) {
        assert_err_eq!(ComponentName::from_str(token_str), InvalidComponentName {});
    }

    #[rstest]
    #[case("abc")]
    #[case("rel*")]
    #[case("a.bc")]
    #[case("a_.-+bc")]
    #[case("a#bc")]
    #[case("ab|c1")]
    #[case("content-type")]
    #[case("@method")]
    #[case("@request-target")]
    #[case("link")]
    fn valid_comp_name_will_be_parsed(#[case] token_str: &str) {
        assert_ok!(ComponentName::from_str(token_str));
    }
}
