//! I define [`IsNonDot`] predicates over a `Segment`.

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::uri::component::segment::SegmentStr;

/// A predicate about a [`SegmentStr`] asserting that it is not a dot segment.
#[derive(Debug)]
pub struct IsNonDot;

impl Predicate<SegmentStr> for IsNonDot {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsNonDot".into()
    }
}

impl PurePredicate<SegmentStr> for IsNonDot {}

impl SyncEvaluablePredicate<SegmentStr> for IsNonDot {
    type EvalError = NotANonDotSegmentStr;

    #[inline]
    fn evaluate_for(sub: &SegmentStr) -> Result<(), Self::EvalError> {
        match [".", ".."].contains(&sub.as_ref()) {
            true => Err(NotANonDotSegmentStr),
            false => Ok(()),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Given segment str is not a non-dot segment str")]
/// Error of Given segment not being a non-dot segment.
pub struct NotANonDotSegmentStr;

#[cfg(test)]
mod tests_evaluation {
    use claims::*;
    use gdp_rs::Proven;
    use rstest::rstest;

    use super::*;

    pub fn try_non_dot_proven_segment_str(
        segment_str: &str,
    ) -> Result<Proven<SegmentStr, IsNonDot>, NotANonDotSegmentStr> {
        Proven::<_, IsNonDot>::try_new(SegmentStr::try_from(segment_str).expect("Claimed valid"))
            .map_err(|e| e.error)
    }

    #[rstest]
    #[case::dot(".")]
    #[case::dot2("..")]
    #[trace]
    fn dot_segment_will_be_rejected(#[case] segment_str: &str) {
        assert_err!(try_non_dot_proven_segment_str(segment_str));
    }

    #[rstest]
    #[case::empty("")]
    #[case::dot3("...")]
    #[case("ayodhya")]
    #[trace]
    fn non_dot_segment_will_be_accepted(#[case] segment_str: &'static str) {
        assert_ok!(try_non_dot_proven_segment_str(segment_str));
    }
}
