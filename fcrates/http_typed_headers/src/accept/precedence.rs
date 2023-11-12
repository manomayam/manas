//! I define [`AcceptPrecedence`] struct.
//!
use mime::Mime;

use crate::common::qvalue::QValue;

/// This enum denotes specificity of a given media range
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaRangeSpecificity {
    /// Specificity of `*/*`
    STAR_STAR,
    /// Specificity of `<type>/*`
    TYPE_STAR,
    /// Specificity of `<type>/<subtype>`, with optional number of params
    EXACT {
        /// Number of parameters in media range.
        param_count: usize,
    },
}

impl From<&Mime> for MediaRangeSpecificity {
    #[inline]
    fn from(media_range: &Mime) -> Self {
        if media_range.type_() == mime::STAR {
            Self::STAR_STAR
        } else if media_range.subtype() == mime::STAR {
            Self::TYPE_STAR
        } else {
            Self::EXACT {
                param_count: media_range.params().count(), // NOTE param 'q' also counted
            }
        }
    }
}

/// A structure that denotes resolved precedence score of an accept-value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AcceptPrecedence {
    /// Weight associated with an accept value.
    pub weight: QValue,

    /// Resolved media-range-specificity for an accept-value.
    pub media_range_specificity: MediaRangeSpecificity,
}

/// Tests [`MediaTypeSpecificity`]
#[cfg(test)]
mod tests_media_range_specificity {
    use std::cmp::Ordering;

    use mime::Mime;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::star_star(mime::STAR_STAR, MediaRangeSpecificity::STAR_STAR)]
    #[case::type_star1(mime::IMAGE_STAR, MediaRangeSpecificity::TYPE_STAR)]
    #[case::type_star2(mime::TEXT_STAR, MediaRangeSpecificity::TYPE_STAR)]
    #[case::exact1(mime::IMAGE_PNG, MediaRangeSpecificity::EXACT { param_count: 0 })]
    #[case::exact1(mime::TEXT_CSS_UTF_8, MediaRangeSpecificity::EXACT { param_count: 1 })]
    fn parse_works_correctly(#[case] media_type: Mime, #[case] expected: MediaRangeSpecificity) {
        assert_eq!(MediaRangeSpecificity::from(&media_type), expected);
    }

    #[rstest]
    #[case(
        MediaRangeSpecificity::STAR_STAR,
        MediaRangeSpecificity::STAR_STAR,
        Ordering::Equal
    )]
    #[case(
        MediaRangeSpecificity::STAR_STAR,
        MediaRangeSpecificity::TYPE_STAR,
        Ordering::Less
    )]
    #[case(
        MediaRangeSpecificity::STAR_STAR,
        MediaRangeSpecificity::EXACT { param_count: 0 },
        Ordering::Less
    )]
    #[case(
        MediaRangeSpecificity::TYPE_STAR,
        MediaRangeSpecificity::STAR_STAR,
        Ordering::Greater
    )]
    #[case(
        MediaRangeSpecificity::TYPE_STAR,
        MediaRangeSpecificity::TYPE_STAR,
        Ordering::Equal
    )]
    #[case(
        MediaRangeSpecificity::TYPE_STAR,
        MediaRangeSpecificity::EXACT { param_count: 0 },
        Ordering::Less
    )]
    #[case(
        MediaRangeSpecificity::EXACT { param_count: 0 },
        MediaRangeSpecificity::STAR_STAR,
        Ordering::Greater
    )]
    #[case(
        MediaRangeSpecificity::EXACT { param_count: 0 },
        MediaRangeSpecificity::TYPE_STAR,
        Ordering::Greater
    )]
    #[case(
        MediaRangeSpecificity::EXACT { param_count: 0 },
        MediaRangeSpecificity::EXACT { param_count: 0 },
        Ordering::Equal
    )]
    #[case(
        MediaRangeSpecificity::EXACT { param_count: 1 },
        MediaRangeSpecificity::EXACT { param_count: 0 },
        Ordering::Greater
    )]
    #[case(
        MediaRangeSpecificity::EXACT { param_count: 0 },
        MediaRangeSpecificity::EXACT { param_count: 1 },
        Ordering::Less
    )]
    fn ordering_works_correctly(
        #[case] mts1: MediaRangeSpecificity,
        #[case] mts2: MediaRangeSpecificity,
        #[case] expected: Ordering,
    ) {
        assert_eq!(mts1.cmp(&mts2), expected);
    }
}

#[cfg(test)]
mod tests_media_range_precedence {
    use std::{cmp::Ordering, str::FromStr};

    use rstest::rstest;

    use super::*;

    fn parse_valid(
        qvalue_str: &str,
        media_type_specificity: MediaRangeSpecificity,
    ) -> AcceptPrecedence {
        AcceptPrecedence {
            weight: QValue::from_str(qvalue_str).unwrap(),
            media_range_specificity: media_type_specificity,
        }
    }

    #[rstest]
    #[case(
        parse_valid("0", MediaRangeSpecificity::EXACT { param_count: 0 }),
        parse_valid("0", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Equal
    )]
    #[case(
        parse_valid("0", MediaRangeSpecificity::EXACT { param_count: 0 }),
        parse_valid("0", MediaRangeSpecificity::EXACT { param_count: 1 }),
        Ordering::Less
    )]
    #[case(
        parse_valid("0", MediaRangeSpecificity::EXACT { param_count: 1 }),
        parse_valid("0", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Greater
    )]
    #[case(
        parse_valid("0.12", MediaRangeSpecificity::TYPE_STAR),
        parse_valid("0.12", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Less
    )]
    #[case(
        parse_valid("1", MediaRangeSpecificity::TYPE_STAR),
        parse_valid("1", MediaRangeSpecificity::STAR_STAR),
        Ordering::Greater
    )]
    #[case(
        parse_valid("0.12", MediaRangeSpecificity::EXACT { param_count: 0 }),
        parse_valid("0.23", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Less
    )]
    #[case(
        parse_valid("0.12", MediaRangeSpecificity::EXACT { param_count: 1 }),
        parse_valid("0.23", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Less
    )]
    #[case(
        parse_valid("0.378", MediaRangeSpecificity::TYPE_STAR),
        parse_valid("1", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Less
    )]
    #[case(
        parse_valid("0", MediaRangeSpecificity::TYPE_STAR),
        parse_valid("1", MediaRangeSpecificity::STAR_STAR),
        Ordering::Less
    )]
    #[case(
        parse_valid("0.345", MediaRangeSpecificity::EXACT { param_count: 0 }),
        parse_valid("0.34", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Greater
    )]
    #[case(
        parse_valid("0.345", MediaRangeSpecificity::EXACT { param_count: 0 }),
        parse_valid("0.34", MediaRangeSpecificity::EXACT { param_count: 1 }),
        Ordering::Greater
    )]
    #[case(
        parse_valid("0.12", MediaRangeSpecificity::TYPE_STAR),
        parse_valid("0", MediaRangeSpecificity::EXACT { param_count: 0 }),
        Ordering::Greater
    )]
    #[case(
        parse_valid("1", MediaRangeSpecificity::TYPE_STAR),
        parse_valid("0", MediaRangeSpecificity::STAR_STAR),
        Ordering::Greater
    )]
    fn ordering_works_correctly(
        #[case] p1: AcceptPrecedence,
        #[case] p2: AcceptPrecedence,
        #[case] expected: Ordering,
    ) {
        assert_eq!(p1.cmp(&p2), expected);
    }
}
