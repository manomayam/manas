//! I define [LinkRel`] struct.
//!
use once_cell::sync::Lazy;
use smallvec::SmallVec;
use vec1::smallvec_v1::SmallVec1;

use super::relation_type::{InvalidEncodedRelationType, RelationType};
use crate::common::field::rules::parameter_name::FieldParameterName;

/// Constant for `rel` param name.
pub static REL_PARAM_NAME: Lazy<FieldParameterName> =
    Lazy::new(|| "rel".parse().expect("Must be valid"));

/// Rel as defined in [`rfc8288`````````````````](https://datatracker.ietf.org/doc/html/rfc8288#section-3.3)
#[derive(Debug, Clone)]
pub struct LinkRel {
    /// List of one or mor relation-types.
    pub rel_types: SmallVec1<[RelationType; 1]>,
}

#[derive(Debug, Clone, thiserror::Error)]
/// Error of invalid encoded link-rel.
pub enum InvalidEncodedLinkRel {
    /// Is Empty.
    #[error("Rel is empty")]
    IsEmpty,

    /// Invalid relation type.
    #[error("relation-type is invalid")]
    InvalidRelationType(#[from] InvalidEncodedRelationType),
}

impl LinkRel {
    /// Decodes header encoded link rel
    pub fn decode(encoded_str: &str) -> Result<Self, InvalidEncodedLinkRel> {
        let mut rel_types = SmallVec::new();
        for rel_type_str in encoded_str.trim().split_ascii_whitespace() {
            rel_types.push(rel_type_str.parse()?);
        }

        Ok(Self {
            rel_types: SmallVec1::try_from_smallvec(rel_types)
                .map_err(|_| InvalidEncodedLinkRel::IsEmpty)?,
        })
    }

    /// Push encoded value to buffer
    #[inline]
    pub(crate) fn push_encoded_str(&self, buffer: &mut String) {
        for (i, rel_type) in self.rel_types.iter().enumerate() {
            if i > 0 {
                buffer.push(' ');
            }
            rel_type.push_encoded_str(buffer);
        }
    }

    /// Encode this rel as str
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }

    /// Get new link rel, with specified first_rel_type.
    #[inline]
    pub fn new(first_rel_type: RelationType) -> Self {
        Self {
            rel_types: SmallVec1::new(first_rel_type),
        }
    }
}

#[cfg(test)]
mod tests_decode {
    use claims::*;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("type", &["type"])]
    #[case("type hreflang", &["type", "hreflang"])]
    #[case("title* http://purl.org/dc/title", &["title*", "http://purl.org/dc/title"])]
    #[case("http://purl.org/dc/title", &["http://purl.org/dc/title"])]
    fn valid_rel_will_be_decoded_correctly(
        #[case] encoded_rel_str: &str,
        #[case] expected_rel_types: &[&str],
    ) {
        let rel = assert_ok!(LinkRel::decode(encoded_rel_str));
        let rel_types = &rel.rel_types;
        assert_eq!(
            rel_types.len(),
            expected_rel_types.len(),
            "rel_types cardinality mismatch."
        );

        for rel_type in rel_types {
            assert!(
                expected_rel_types.contains(&rel_type.str_encode().as_str()),
                "rel_type \"{}\" is not expected",
                rel_type.str_encode().as_str()
            );
        }
    }

    #[test]
    fn empty_rel_will_be_rejected() {
        assert_matches!(
            assert_err!(LinkRel::decode(" ")),
            InvalidEncodedLinkRel::IsEmpty
        );
    }

    #[rstest]
    #[case("a/b")]
    #[case("a{b")]
    fn rel_with_invalid_rel_type_will_be_rejected(#[case] rel_str: &str) {
        assert_matches!(
            assert_err!(LinkRel::decode(rel_str)),
            InvalidEncodedLinkRel::InvalidRelationType(..)
        );
    }
}

#[cfg(test)]
mod tests_encode {
    use claims::*;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("type", "type")]
    #[case("type hreflang", "type hreflang")]
    #[case("Type hreflang", "type hreflang")]
    #[case("title* http://purl.org/dc/title", "title* http://purl.org/dc/title")]
    #[case("http://purl.org/dc/title", "http://purl.org/dc/title")]
    fn rel_decode_encode_round_trip_works_correctly(#[case] rel_str: &str, #[case] expected: &str) {
        let rel = assert_ok!(LinkRel::decode(rel_str));
        assert_eq!(rel.str_encode().as_str(), expected);
    }
}
