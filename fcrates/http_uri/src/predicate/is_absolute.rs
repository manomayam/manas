//! I define [`IsAbsolute`] predicate over `HttpUri`.

use std::borrow::Cow;

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::HttpUri;

/// A predicate about an [`HttpUri`] asserting that it is in absolute form.
#[derive(Debug)]
pub struct IsAbsolute;

impl Predicate<HttpUri> for IsAbsolute {
    fn label() -> Cow<'static, str> {
        "IsAbsolute".into()
    }
}

impl PurePredicate<HttpUri> for IsAbsolute {}

impl SyncEvaluablePredicate<HttpUri> for IsAbsolute {
    type EvalError = NotAnAbsoluteHttpUri;

    #[inline]
    fn evaluate_for(sub: &HttpUri) -> Result<(), Self::EvalError> {
        match sub.fragment() {
            Some(_) => Err(NotAnAbsoluteHttpUri),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of HttpUri being not an absolute uri.
#[error("Given http uri is not an absolute http uri")]
pub struct NotAnAbsoluteHttpUri;

#[cfg(test)]
mod tests_evaluation {
    use claims::*;
    use gdp_rs::proven::Proven;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case::with_fragment("http://pod1.example.org/a#bc")]
    #[case::with_query_fragment("http://pod1.example.org/a?b#c")]
    fn uri_with_fragment_will_be_rejected(#[case] uri_str: &'static str) {
        let uri = HttpUri::try_from(uri_str).expect("Claimed valid uri str");
        assert_err_eq!(
            Proven::<_, IsAbsolute>::try_new(uri).map_err(|e| e.error),
            NotAnAbsoluteHttpUri
        );
    }

    #[rstest]
    #[case("http://pod1.example.org/a")]
    #[case::with_query("http://pod1.example.org/a?b")]
    fn absolute_uri_sans_will_be_accepted(#[case] uri_str: &'static str) {
        let uri = HttpUri::try_from(uri_str).expect("Claimed valid uri str");
        assert_ok!(Proven::<_, IsAbsolute>::try_new(uri),);
    }
}
