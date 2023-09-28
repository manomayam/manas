//! I define [`AcceptValue`] struct to represent single value in `Accept` header.
//!

use std::{ops::Deref, str::FromStr};

use mime::Mime;

use super::precedence::{AcceptPrecedence, MediaRangeSpecificity};
use crate::header::common::qvalue::{QValue, Q_PARAM_NAME};

/// `AcceptValue` as defined in [`rfc9110`](https://www.rfc-editor.org/rfc/rfc9110.html#section-12.5.1)
///
/// ```txt
/// Accept = #( media-range [ weight ] )
///
///   media-range    = ( "*/*"
///                      / ( type "/" "*" )
///                      / ( type "/" subtype )
///                    ) parameters
/// ```
///
/// The asterisk "`*`" character is used to group media types into ranges, with "`*/*`" indicating all media types and "`type/*`" indicating all subtypes of that type.
/// The media-range can include media type parameters that are applicable to that range.
/// Each media-range might be followed by optional applicable media type parameters (e.g., charset), followed by an optional "q" parameter for indicating a relative weight (Section 12.4.2)
///
/// Senders using weights SHOULD send "`q`" last (after all media-range parameters). Recipients SHOULD process any parameter named "`q`" as weight, regardless of parameter ordering.
#[derive(Debug, Clone)]
pub struct AcceptValue {
    media_range: Mime,
    accept_precedence: AcceptPrecedence,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid accept-value.
pub enum InvalidEncodedAcceptValue {
    /// Invalid header encoding.
    #[error("Invalid header encoding")]
    InvalidHeaderEncoding,

    /// Invalid weight of accept-value.
    #[error("Invalid Weight")]
    InvalidWeight,
}

impl FromStr for AcceptValue {
    type Err = InvalidEncodedAcceptValue;

    fn from_str(value_str: &str) -> Result<Self, Self::Err> {
        // Parse as media range.
        // Is valid, as rfc9110 removes accept-ext requirement.
        let media_range: Mime = value_str
            .parse()
            .map_err(|_| InvalidEncodedAcceptValue::InvalidHeaderEncoding)?;

        // Resolve effective weight
        let weight: QValue =
            if let Some(q_value) = media_range.get_param(Q_PARAM_NAME.as_token().as_ref()) {
                q_value
                    .as_str()
                    .parse()
                    .map_err(|_| InvalidEncodedAcceptValue::InvalidWeight)?
            } else {
                QValue::DEFAULT
            };

        // Resolve accept precedence.
        let accept_precedence = AcceptPrecedence {
            weight,
            media_range_specificity: MediaRangeSpecificity::from(&media_range),
        };

        Ok(Self {
            media_range,
            accept_precedence,
        })
    }
}

impl Deref for AcceptValue {
    type Target = Mime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.media_range
    }
}

impl AcceptValue {
    /// Get precedence of this accept_value.
    #[inline]
    pub fn precedence(&self) -> &AcceptPrecedence {
        &self.accept_precedence
    }

    /// Get weight of this accept_value
    #[inline]
    pub fn weight(&self) -> &QValue {
        &self.accept_precedence.weight
    }

    #[inline]
    /// Get media range of this value.
    pub fn media_range(&self) -> &Mime {
        &self.media_range
    }

    /// Check if a media_type matches this media range.
    pub fn matches(&self, media_type: &Mime, consider_params: bool) -> bool {
        match self.accept_precedence.media_range_specificity {
            // STAR_STAR matches all media_types
            MediaRangeSpecificity::STAR_STAR => true,
            // Matches if type is same
            MediaRangeSpecificity::TYPE_STAR => self.media_range.type_() == media_type.type_(),
            // Matches if essence is same and params also matches
            MediaRangeSpecificity::EXACT { .. } => {
                (media_type.essence_str() == self.media_range.essence_str())
                    && (!consider_params || {
                        self.media_range
                            .params()
                            .all(|(k, v)| k == "q" || Some(v) == media_type.get_param(k))
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests_accept_value_parse {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("text")]
    #[case("text / html")]
    fn accept_value_with_invalid_media_range_will_be_rejected(#[case] accept_value_str: &str) {
        assert_err_eq!(
            AcceptValue::from_str(accept_value_str),
            InvalidEncodedAcceptValue::InvalidHeaderEncoding
        );
    }

    #[rstest]
    #[case::non_number_qvalue("image/png; q=1-2345")]
    #[case::out_of_range_qvalue("image/png; q=2.3")]
    #[case::out_of_scale_qvalue("*/*; q=0.3333")]
    fn accept_value_with_invalid_qvalue_will_be_rejected(#[case] accept_value_str: &str) {
        assert_err_eq!(
            AcceptValue::from_str(accept_value_str),
            InvalidEncodedAcceptValue::InvalidWeight
        );
    }

    fn precedence(
        qvalue_str: &str,
        media_type_specificity: MediaRangeSpecificity,
    ) -> AcceptPrecedence {
        AcceptPrecedence {
            weight: QValue::from_str(qvalue_str).unwrap(),
            media_range_specificity: media_type_specificity,
        }
    }

    pub fn assert_valid_accept_value(value: &str) -> AcceptValue {
        assert_ok!(AcceptValue::from_str(value))
    }

    #[rstest]
    #[case::star_star_default("*/*", precedence("1", MediaRangeSpecificity::STAR_STAR))]
    #[case::star_star_explicit(
        "*/*; q=0.321",
        precedence("0.321", MediaRangeSpecificity::STAR_STAR)
    )]
    #[case::type_star_default("image/*", precedence("1", MediaRangeSpecificity::TYPE_STAR))]
    #[case::type_star_explicit(
        "text/*; q=0.5; charset=utf8",
        precedence("0.5", MediaRangeSpecificity::TYPE_STAR)
    )]
    #[case::exact_default("text/html; charset=utf8", precedence("1", MediaRangeSpecificity::EXACT { param_count: 1 }))]
    #[case::exact_explicit("image/png; q=0.7", precedence("0.7", MediaRangeSpecificity::EXACT { param_count: 1 }))]
    fn valid_accept_value_will_be_parsed_correctly(
        #[case] accept_value_str: &str,
        #[case] expected_specificity: AcceptPrecedence,
    ) {
        let accept_value = assert_valid_accept_value(accept_value_str);

        assert_eq!(accept_value.precedence(), &expected_specificity);
    }
}

#[cfg(test)]
mod tests_accept_value {
    use std::{cmp::Ordering, str::FromStr};

    use rstest::rstest;

    use super::{tests_accept_value_parse::*, *};

    #[rstest]
    #[case::star_star_1("*/*", "image/png", false, true)]
    #[case::type_star_1("image/*", "image/png", false, true)]
    #[case::type_star_2("text/*", "image/png", false, false)]
    #[case::exact_1("image/png", "image/png", false, true)]
    #[case::exact_2("image/jpg", "image/png", false, false)]
    #[case::exact_params1("text/html; q=1; charset=utf8", "text/html", false, true)]
    #[case::exact_params2("text/html; q=1; charset=utf8", "text/html", true, false)]
    #[case::exact_params2("text/html; charset=utf8; q=1", "text/html", true, false)]
    #[case::exact_params3("text/html; q=1; charset=utf8", "text/html; charset=utf8", true, true)]
    #[case::exact_params4("text/html; q=1; charset=utf8", "text/html; charset=UTF8", true, true)]
    #[case::exact_params5("application/json", "application/ld+json", true, false)]
    #[case::with_suffix5("application/ld+json", "application/ld+json", true, true)]
    #[case("application/ld+json", "application/json", true, false)]
    fn matches_works_correctly(
        #[case] accept_value_str: &str,
        #[case] media_range_str: &str,
        #[case] consider_params: bool,
        #[case] expected: bool,
    ) {
        let accept_value = assert_valid_accept_value(accept_value_str);
        let media_type = Mime::from_str(media_range_str).unwrap();

        assert_eq!(accept_value.matches(&media_type, consider_params), expected);
    }

    #[rstest]
    #[case("*/*", "*/*", Ordering::Equal)]
    #[case("*/*; q=0.3", "image/*; q=0.3", Ordering::Less)]
    #[case("*/*", "image/png", Ordering::Less)]
    #[case("image/*", "*/*", Ordering::Greater)]
    #[case("image/*; q=1", "text/*", Ordering::Equal)]
    #[case("image/*", "text/html", Ordering::Less)]
    #[case("image/png", "text/*", Ordering::Greater)]
    #[case("text/html", "application/*", Ordering::Greater)]
    #[case("image/png", "text/html", Ordering::Equal)]
    #[case("text/plain;format=flowed", "text/plain", Ordering::Greater)]
    #[case("*/*; q=0.3", "image/*; q=0.2", Ordering::Greater)]
    #[case("image/png; q=0.5", "text/*; q=0.8", Ordering::Less)]
    fn precedence_ordering_works_correctly(
        #[case] accept_value1_str: &str,
        #[case] accept_value2_str: &str,
        #[case] expected: Ordering,
    ) {
        let accept_value1 = assert_valid_accept_value(accept_value1_str);
        let accept_value2 = assert_valid_accept_value(accept_value2_str);

        assert_eq!(
            accept_value1.precedence().cmp(accept_value2.precedence()),
            expected
        );
    }
}
