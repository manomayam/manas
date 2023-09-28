//! I define [`HasNoQuery`] predicate over `HttpUri`.

use std::borrow::Cow;

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::HttpUri;

/// A predicate about an [`HttpUri`] asserting that it has no query..
#[derive(Debug)]
pub struct HasNoQuery;

impl Predicate<HttpUri> for HasNoQuery {
    fn label() -> Cow<'static, str> {
        "HasNoQuery".into()
    }
}

impl PurePredicate<HttpUri> for HasNoQuery {}

impl SyncEvaluablePredicate<HttpUri> for HasNoQuery {
    type EvalError = HasInvalidQuery;

    #[inline]
    fn evaluate_for(sub: &HttpUri) -> Result<(), Self::EvalError> {
        match sub.query() {
            Some(_) => Err(HasInvalidQuery),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of HttpUri having invalid query.
#[error("Given http uri has invalid query.")]
pub struct HasInvalidQuery;

#[cfg(test)]
mod tests_evaluation {
    use claims::*;
    use gdp_rs::proven::Proven;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case::with_query("http://pod1.example.org/a?")]
    #[case::with_query("http://pod1.example.org/a?bc")]
    #[case::with_query_fragment("http://pod1.example.org/a?b#c")]
    fn uri_with_query_will_be_rejected(#[case] uri_str: &'static str) {
        let uri = HttpUri::try_from(uri_str).expect("Claimed valid uri str");
        assert_err_eq!(
            Proven::<_, HasNoQuery>::try_new(uri).map_err(|e| e.error),
            HasInvalidQuery
        );
    }

    #[rstest]
    #[case("http://pod1.example.org/a")]
    #[case::with_fragment("http://pod1.example.org/a#b")]
    fn uri_sans_query_will_be_accepted(#[case] uri_str: &'static str) {
        let uri = HttpUri::try_from(uri_str).expect("Claimed valid uri str");
        assert_ok!(Proven::<_, HasNoQuery>::try_new(uri),);
    }
}
