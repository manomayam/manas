//! I define [`IsNormal`] predicates over a `Segment`.

use std::{borrow::Cow, marker::PhantomData};

use gdp_rs::{
    inference_rule::{AuthorizedInferenceRuleGhost, InferenceRule, Operation},
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};
use percent_encoding::utf8_percent_encode;
use uriparse::Segment;

use crate::uri::component::{character::pchar::PCHAR_PCT_ENCODE_SET, segment::SegmentStr};

/// A predicate about a [`SegmentStr`] asserting that it is in normal form.
#[derive(Debug)]
pub struct IsNormal;

impl Predicate<SegmentStr> for IsNormal {
    fn label() -> Cow<'static, str> {
        "IsNormal".into()
    }
}

impl PurePredicate<SegmentStr> for IsNormal {}

impl SyncEvaluablePredicate<SegmentStr> for IsNormal {
    type EvalError = NotANormalSegmentStr;

    #[inline]
    fn evaluate_for(sub: &SegmentStr) -> Result<(), Self::EvalError> {
        match sub.as_parsed().is_normalized() {
            true => Ok(()),
            false => Err(NotANormalSegmentStr),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Given segment str is not a normal segment str")]
/// Error of Given segment not being a normal segment.
pub struct NotANormalSegmentStr;

/// An inference rule, that infers normalized [`SegmentStr`] from a given [`Segment`].
pub struct ParsedNormalization<'s, SP> {
    _phantom: PhantomData<fn(&'s SP)>,
}

/// A struct representing normalization transform over a subject [`Segment`].
#[derive(Debug, Default)]
pub struct ParsedNormalizationTransform<'s> {
    _phantom: PhantomData<fn(Segment<'s>)>,
}

impl<'s> Operation for ParsedNormalizationTransform<'s> {
    type Arg = Segment<'s>;

    type Result = SegmentStr;

    #[inline]
    fn call(self, mut source_sub: Self::Arg) -> Self::Result {
        source_sub.normalize();
        SegmentStr::from(source_sub)
    }
}

impl<'s, SP: Predicate<Segment<'s>>> InferenceRule for ParsedNormalization<'s, SP> {
    type SourceSub = Segment<'s>;

    type SourcePredicate = SP;

    type TargetSub = SegmentStr;

    type TargetPredicate = IsNormal;

    type SubjectTransform = ParsedNormalizationTransform<'s>;
}

impl<'s, SP: Predicate<SegmentStr>> AuthorizedInferenceRuleGhost<IsNormal, SegmentStr>
    for PhantomData<ParsedNormalization<'s, SP>>
{
}

/// An inference rule, that infers pct-encoded, normalized [`SegmentStr`] from a given [`&str`].
#[derive(Debug, Default)]
pub struct PctEncodingNormalization<'s, SP> {
    _phantom: PhantomData<fn(&'s SP)>,
}

/// A struct representing pct-encoding normalization transform over a subject [`&str].
#[derive(Debug, Default)]
pub struct PctEncodingNormalizationTransform<'s> {
    _phantom: PhantomData<fn(&'s str)>,
}

impl<'s> Operation for PctEncodingNormalizationTransform<'s> {
    type Arg = &'s str;

    type Result = SegmentStr;

    #[inline]
    fn call(self, source_sub: Self::Arg) -> Self::Result {
        // First create valid segment, after percent encoding source string.
        let resolved_segment = if let Cow::Owned(encoded_segment_str) =
            utf8_percent_encode(source_sub, PCHAR_PCT_ENCODE_SET).into()
        {
            let segment =
                Segment::try_from(encoded_segment_str.as_str()).expect("Must be valid segment.");
            segment.into_owned()
        } else {
            Segment::try_from(source_sub).expect("Must be valid segment.")
        };

        // Then normalize resultant segment.
        ParsedNormalizationTransform::default().call(resolved_segment)
    }
}

impl<'s, SP: Predicate<&'s str>> InferenceRule for PctEncodingNormalization<'s, SP> {
    type SourceSub = &'s str;

    type SourcePredicate = SP;

    type TargetSub = SegmentStr;

    type TargetPredicate = IsNormal;

    type SubjectTransform = PctEncodingNormalizationTransform<'s>;
}

impl<'s, SP: Predicate<&'s str>> AuthorizedInferenceRuleGhost<IsNormal, SegmentStr>
    for PhantomData<PctEncodingNormalization<'s, SP>>
{
}

#[cfg(test)]
mod tests_evaluation {
    use claims::*;
    use gdp_rs::Proven;
    use rstest::rstest;

    use super::*;

    pub fn try_normal_proven_segment_str(
        segment_str: &str,
    ) -> Result<Proven<SegmentStr, IsNormal>, NotANormalSegmentStr> {
        Proven::<_, IsNormal>::try_new(SegmentStr::try_from(segment_str).expect("Claimed valid"))
            .map_err(|e| e.error)
    }

    #[rstest]
    #[case::a("%61yodhya")]
    #[case::a_cap("%41yodhya")]
    #[case::dig_1("raghu%31")]
    #[case::hyphen("rama%2Drajya")]
    #[case::tilde("rama%7Esita")]
    #[case::period("kosala%2Eayodhya")]
    #[case::period_2("%2E%2E")]
    #[case::underscore("rama%5Flakshmana")]
    #[trace]
    fn un_reserved_char_encoded_segment_will_be_rejected(#[case] segment_str: &'static str) {
        assert_err!(try_normal_proven_segment_str(segment_str));
    }

    #[rstest]
    #[case("rama%3ddharma")]
    #[case("rama%2blakshmana")]
    #[case("rama%2clakshmana")]
    #[case("%e0%A4%b0%E0%A4%BE%E0%A4%AE")]
    #[case("kosala%2fayodhya")]
    #[trace]
    fn lowercase_pct_encoded_segment_will_be_rejected(#[case] segment_str: &'static str) {
        assert_err!(try_normal_proven_segment_str(segment_str));
    }

    #[rstest]
    #[case::empty("")]
    #[case::dot(".")]
    #[case::dot2("..")]
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
    #[case::expected_gen_delim_unencoded_2("a@b")]
    #[case::excepted_gen_delim_encoded_1("a%3Ab")]
    #[case::expected_gen_delim_encoded_2("a%40b")]
    #[case::gen_delim_encoded_1("b%2Fc")]
    #[case::gen_delim_encoded_2("b%3Fc")]
    #[case::gen_delim_encoded_3("b%23c")]
    #[case::gen_delim_encoded_4("b%5B%5Dc")]
    #[case::non_ascii_pct_encoded("%E0%A4%B0%E0%A4%BE%E0%A4%AE")]
    #[case::non_ascii_pct_encoded("%E0%B0%85%E0%B0%AF%E0%B1%8B%E0%B0%A7%E0%B1%8D%E0%B0%AF")]
    #[trace]
    fn normalized_segment_will_be_accepted(#[case] segment_str: &'static str) {
        assert_ok!(try_normal_proven_segment_str(segment_str));
    }
}

#[cfg(test)]
mod tests_normalization {
    use gdp_rs::Proven;
    use rstest::*;

    use super::*;

    // Asserts normalization expectation.
    pub fn assert_correct_normalization(source_segment_str: &str, expected_segment_str: &str) {
        let source_segment =
            Segment::try_from(source_segment_str).expect("Claimed valid segment str");
        let pr_segment =
            Proven::void_proven(source_segment).infer::<ParsedNormalization<_>>(Default::default());
        assert_eq!(pr_segment.as_ref().as_ref(), expected_segment_str);
    }

    #[rstest]
    #[case::a("%61yodhya", "ayodhya")]
    #[case::a_cap("%41yodhya", "Ayodhya")]
    #[case::dig_1("raghu%31", "raghu1")]
    #[case::hyphen("rama%2Drajya", "rama-rajya")]
    #[case::tilde("rama%7Esita", "rama~sita")]
    #[case::period("kosala%2Eayodhya", "kosala.ayodhya")]
    #[case::period_2("%2E%2E", "..")]
    #[case::underscore("rama%5Flakshmana", "rama_lakshmana")]
    #[trace]
    fn unreserved_chars_will_be_normalized_correctly(
        #[case] source_segment_str: &'static str,
        #[case] expected_segment_str: &'static str,
    ) {
        assert_correct_normalization(source_segment_str, expected_segment_str);
    }

    #[rstest]
    #[case("rama%3ddharma", "rama%3Ddharma")]
    #[case("rama%2blakshmana", "rama%2Blakshmana")]
    #[case("rama%2clakshmana", "rama%2Clakshmana")]
    #[case("%e0%A4%b0%E0%A4%BE%E0%A4%AE", "%E0%A4%B0%E0%A4%BE%E0%A4%AE")]
    #[case("kosala%2fayodhya", "kosala%2Fayodhya")]
    #[trace]
    fn pct_encoded_octet_case_will_be_normalized_correctly(
        #[case] source_segment_str: &'static str,
        #[case] expected_segment_str: &'static str,
    ) {
        assert_correct_normalization(source_segment_str, expected_segment_str);
    }
}

#[cfg(test)]
mod tests_encoding_normalization {
    use gdp_rs::Proven;
    use rstest::*;

    use super::*;

    pub fn assert_correct_normalization(
        source_segment_str: &'static str,
        expected_segment_str: &'static str,
    ) {
        let pr_segment = Proven::void_proven(source_segment_str)
            .infer::<PctEncodingNormalization<_>>(Default::default());
        assert_eq!(pr_segment.as_ref().as_ref(), expected_segment_str);
    }

    #[rstest]
    #[case::invalid_char1("a b", "a%20b")]
    #[case::invalid_char2("a\nb", "a%0Ab")]
    #[case::invalid_char3("a\\b", "a%5Cb")]
    #[case::invalid_char3("a%b", "a%25b")]
    #[case::invalid_char4("rama<>sita/doc", "rama%3C%3Esita%2Fdoc")]
    #[case::invalid_non_ascii1("राम", "%E0%A4%B0%E0%A4%BE%E0%A4%AE")]
    #[case::invalid_non_ascii2("అయోధ్య", "%E0%B0%85%E0%B0%AF%E0%B1%8B%E0%B0%A7%E0%B1%8D%E0%B0%AF")]
    #[case::invalid_gen_delim3("a/b[c", "a%2Fb%5Bc")]
    #[case::invalid_gen_delim4("a/b]c", "a%2Fb%5Dc")]
    #[case::uri(
        "http://pod1.example.org/path/to/a",
        "http:%2F%2Fpod1.example.org%2Fpath%2Fto%2Fa"
    )]
    #[trace]
    fn segment_str_will_be_properly_pct_encoded_and_normalized(
        #[case] source_segment_str: &'static str,
        #[case] expected_segment_str: &'static str,
    ) {
        assert_correct_normalization(source_segment_str, expected_segment_str);
    }
}
