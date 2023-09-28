//! I define [`IsNonEmpty`] predicates over a `Segment`.

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::uri::component::segment::SegmentStr;

/// A predicate about a [`SegmentStr`] asserting that it is not an empty segment.
#[derive(Debug)]
pub struct IsNonEmpty;

impl Predicate<SegmentStr> for IsNonEmpty {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsNonEmpty".into()
    }
}

impl PurePredicate<SegmentStr> for IsNonEmpty {}

impl SyncEvaluablePredicate<SegmentStr> for IsNonEmpty {
    type EvalError = NotANonEmptySegmentStr;

    #[inline]
    fn evaluate_for(sub: &SegmentStr) -> Result<(), Self::EvalError> {
        match sub.is_empty() {
            true => Err(NotANonEmptySegmentStr),
            false => Ok(()),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Given segment str is not a non-empty segment str")]
/// Error of Given segment not being a non-empty segment.
pub struct NotANonEmptySegmentStr;

#[cfg(test)]
mod tests_evaluation {
    use claims::*;
    use gdp_rs::Proven;
    use rstest::rstest;

    use super::*;

    pub fn try_non_empty_proven_segment_str(
        segment_str: &str,
    ) -> Result<Proven<SegmentStr, IsNonEmpty>, NotANonEmptySegmentStr> {
        Proven::<_, IsNonEmpty>::try_new(SegmentStr::try_from(segment_str).expect("Claimed valid"))
            .map_err(|e| e.error)
    }

    #[rstest]
    #[case::empty("")]
    #[trace]
    fn empty_segment_will_be_rejected(#[case] segment_str: &str) {
        assert_err!(try_non_empty_proven_segment_str(segment_str));
    }

    #[rstest]
    #[case::dot1(".")]
    #[case::dot2("..")]
    #[case("ayodhya")]
    #[trace]
    fn non_empty_segment_will_be_accepted(#[case] segment_str: &'static str) {
        assert_ok!(try_non_empty_proven_segment_str(segment_str));
    }
}
