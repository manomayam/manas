//! I define few invariants of `Segment`.
//!
use frunk_core::HList;
use gdp_rs::{predicate::impl_::all_of::AllOf, Proven};

use super::{
    predicate::{is_non_dot::IsNonDot, is_non_empty::IsNonEmpty, is_normal::IsNormal},
    SegmentStr,
};

/// Type alias for a normal segment str.
pub type NormalSegmentStr = Proven<SegmentStr, AllOf<SegmentStr, HList!(IsNormal)>>;

/// Type alias for a non-empty, normal segment str.
pub type NonEmptyNormalSegmentStr =
    Proven<SegmentStr, AllOf<SegmentStr, HList!(IsNonEmpty, IsNormal)>>;

/// Type alias for a non-empty clean segment str.
pub type NonEmptyCleanSegmentStr =
    Proven<SegmentStr, AllOf<SegmentStr, HList!(IsNonEmpty, IsNonDot, IsNormal)>>;

#[cfg(test)]
mod tests_nonempty_clean_segment_evaluation {
    use claims::*;
    use rstest::*;

    use super::*;
    use crate::uri::component::segment::predicate::{
        is_non_dot::NotANonDotSegmentStr, is_non_empty::NotANonEmptySegmentStr,
        is_normal::NotANormalSegmentStr,
    };

    pub fn try_non_empty_clean_proven_segment_str(
        segment_str: &str,
    ) -> Result<NonEmptyCleanSegmentStr, Box<dyn std::error::Error + Send + Sync>> {
        NonEmptyCleanSegmentStr::try_new(SegmentStr::try_from(segment_str).expect("Claimed valid"))
            .map_err(|e| e.error)
    }

    #[test]
    fn empty_segment_will_be_rejected() {
        assert_err!(try_non_empty_clean_proven_segment_str(""))
            .downcast::<NotANonEmptySegmentStr>()
            .expect("Unexpected error type in compound predicate evaluation.");
    }

    #[rstest]
    #[case("rama%3ddharma")]
    #[case("rama%2blakshmana")]
    #[case::a("%61yodhya")]
    #[case::a_cap("%41yodhya")]
    #[case::dig_1("raghu%31")]
    #[case::hyphen("rama%2Drajya")]
    #[case::tilde("rama%7Esita")]

    fn non_normal_segment_will_be_rejected(#[case] segment_str: &str) {
        assert_err!(try_non_empty_clean_proven_segment_str(segment_str),)
            .downcast::<NotANormalSegmentStr>()
            .expect("Unexpected error type in compound predicate evaluation.");
    }

    #[rstest]
    #[case::dot(".")]
    #[case::dot2("..")]
    fn dot_segment_will_be_rejected(#[case] segment_str: &str) {
        assert_err!(try_non_empty_clean_proven_segment_str(segment_str),)
            .downcast::<NotANonDotSegmentStr>()
            .expect("Unexpected error type in compound predicate evaluation.");
    }

    #[rstest]
    #[case("ayodhya")]
    #[case::un_reserved_unencoded_1("a.acl")]
    #[case::un_reserved_unencoded_2("a~acl")]
    #[case::un_reserved_unencoded_3("a_acl")]
    #[case::un_reserved_unencoded_4("a-acl")]
    #[case::sub_delim_unencoded_1("a$acl")]
    #[case::sub_delim_unencoded_2("rama=dharma")]
    #[case::sub_delim_unencoded_3("rama+lakshmana")]
    #[case::sub_delim_unencoded_4("rama,lakshmana")]
    #[case::sub_delim_unencoded_5("rama&lakshmana")]
    #[case::sub_delim_encoded_1("%24acl")]
    #[case::sub_delim_encoded_2("rama%3Ddharma")]
    #[case::sub_delim_encoded_3("rama%2Blakshmana")]
    #[case::sub_delim_encoded_4("rama%2Clakshmana")]
    #[case::sub_delim_encoded_5("rama%26lakshmana")]
    #[case::excepted_gen_delim_unencoded_1("a:b")]
    #[case::excepted_gen_delim_unencoded_2("a@b")]
    #[case::excepted_gen_delim_encoded_1("a%3Ab")]
    #[case::expected_gen_delim_encoded_2("a%40b")]
    #[case::gen_delim_encoded_1("b%2Fc")]
    #[case::gen_delim_encoded_2("b%3Fc")]
    #[case::gen_delim_encoded_3("b%23c")]
    #[case::gen_delim_encoded_4("b%5B%5Dc")]
    #[case::non_ascii_pct_encoded("%E0%A4%B0%E0%A4%BE%E0%A4%AE")]
    #[case::non_ascii_pct_encoded("%E0%B0%85%E0%B0%AF%E0%B1%8B%E0%B0%A7%E0%B1%8D%E0%B0%AF")]
    fn non_empty_clean_segment_will_be_accepted(#[case] segment_str: &str) {
        assert_ok!(try_non_empty_clean_proven_segment_str(segment_str),);
    }
}
