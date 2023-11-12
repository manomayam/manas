//! I define [`RelationType`] struct.
//! corresponding to `relation-type` production.
//!
use std::{borrow::Borrow, str::FromStr, sync::Arc};

use iri_string::types::UriStr;
use once_cell::sync::Lazy;
use regex::Regex;
use unicase::Ascii;

use crate::common::field::rules::token::Token;

/// A struct or representing arced uri string.
#[derive(Debug, Clone)]
pub struct ArcUriStr(Arc<UriStr>);

impl AsRef<str> for ArcUriStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

/// RelationType is defined in [`rfc8288`](https://datatracker.ietf.org/doc/html/rfc8288#section-3.3)
///
/// ```txt
/// relation-type  = reg-rel-type / ext-rel-type
/// reg-rel-type   = LOALPHA *( LOALPHA / DIGIT / "." / "-" )
/// ext-rel-type   = URI ; Section 3 of [RFC3986]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    /// Registered relation tpe corresponding to `reg-rel-type` production.
    Registered(Ascii<Token>),

    /// Extension relation type corresponding to `ext-rel-type` production.
    Extension(Ascii<ArcUriStr>),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Error of invalid relation type.
pub enum InvalidEncodedRelationType {
    /// Invalid relation type.
    #[error("Invalid relation type source")]
    Invalid,
}

static UP_ASCII_RE: Lazy<Regex> = Lazy::new(|| Regex::new("[A-Z]").expect("Must be valid."));

impl FromStr for RelationType {
    type Err = InvalidEncodedRelationType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check if relation-type is extension kind
        if let Ok(uri) = <&UriStr>::try_from(s) {
            return Ok(Self::Extension(Ascii::new(ArcUriStr(uri.into()))));
        }

        let token_result = if UP_ASCII_RE.is_match(s) {
            // registered tokens must be lowercase, as per spec.
            Token::from_str(&s.to_ascii_lowercase())
        } else {
            Token::from_str(s)
        };

        // Check if relation type is registered kind
        if let Ok(token) = token_result {
            return Ok(Self::Registered(Ascii::new(token)));
        }
        // Otherwise return error
        Err(InvalidEncodedRelationType::Invalid)
    }
}

impl AsRef<str> for RelationType {
    #[inline]
    fn as_ref(&self) -> &str {
        match self {
            RelationType::Registered(token) => token.as_ref(),
            RelationType::Extension(uri) => uri.0.as_str(),
        }
    }
}

impl Borrow<str> for RelationType {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl RelationType {
    /// Decodes header encoded link rel_type
    #[inline]
    pub fn decode(encoded_rel_str: &str) -> Result<Self, InvalidEncodedRelationType> {
        encoded_rel_str.parse()
    }

    /// Push encoded value to string buffer
    #[inline]
    pub(crate) fn push_encoded_str(&self, buffer: &mut String) {
        buffer.push_str(self.as_ref());
    }

    /// Encode relation type as a string as per rfc
    #[inline]
    pub fn str_encode(&self) -> String {
        self.as_ref().to_string()
    }
}

/// Macro to define static [`RelationType`] values.
#[macro_export(local_inner_macros)]
macro_rules! define_static_rel_types {
    (
        $($(#[$outer:meta])*$REL_TYPE:ident: $val:expr;)*
    ) => {
        $(
            #[allow(unused_qualifications)]
            $(#[$outer])*
            pub static $REL_TYPE: once_cell::sync::Lazy<$crate::link::RelationType> = once_cell::sync::Lazy::new(|| {
                $val
                    .parse()
                    .expect("RelationType claimed to be valid")
            });
        )*
    };
}

define_static_rel_types!(
    /// "type" relation type.
    TYPE_REL_TYPE: "type";
);

#[cfg(test)]
mod tests {
    use claims::*;
    use rstest::*;

    use super::*;

    // Define test link relation types
    define_static_rel_types!(
        #[allow(dead_code)]
        ALTERNATE: "alternate";

        #[allow(dead_code)]
        APPENDIX: "appendix";
    );

    #[rstest]
    #[case("type")]
    #[case("hreflang")]
    #[case("service")]
    fn valid_registered_rel_types_will_be_parsed_correctly(#[case] relation_type_str: &str) {
        let relation_type: RelationType = assert_ok!(relation_type_str.parse());
        assert_matches!(relation_type, RelationType::Registered(_));
    }

    #[rstest]
    #[case("a:b")]
    #[case("http://example.org/rel1")]
    #[case("http://example.org/rel2#")]
    fn valid_extension_rel_types_will_be_parsed_correctly(#[case] relation_type_str: &str) {
        let relation_type: RelationType = assert_ok!(relation_type_str.parse());
        assert_matches!(relation_type, RelationType::Extension(_));
    }

    #[rstest]
    #[case("a b")]
    #[trace]
    fn invalid_relation_types_will_be_rejected(#[case] relation_type_str: &str) {
        assert_err!(RelationType::from_str(relation_type_str));
    }
}
