//! I define [`PathHasTrailingSlash`] predicate over `HttpUri`.

use std::borrow::Cow;

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::HttpUri;

/// A predicate about an [`HttpUri`] asserting that it's path
/// has trailing slash.
#[derive(Debug)]
pub struct PathHasTrailingSlash;

impl Predicate<HttpUri> for PathHasTrailingSlash {
    fn label() -> Cow<'static, str> {
        "PathHasTrailingSlash".into()
    }
}

impl PurePredicate<HttpUri> for PathHasTrailingSlash {}

impl SyncEvaluablePredicate<HttpUri> for PathHasTrailingSlash {
    type EvalError = PathHasNoTrailingSlash;

    #[inline]
    fn evaluate_for(sub: &HttpUri) -> Result<(), Self::EvalError> {
        if sub.path_str().ends_with('/') {
            Ok(())
        } else {
            Err(PathHasNoTrailingSlash)
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of HttpUri's path having no trailing slash.
#[error("Given http uri's path has no trailing slash.")]
pub struct PathHasNoTrailingSlash;

#[cfg(test)]
mod tests_evaluation {
    use claims::*;
    use gdp_rs::proven::Proven;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("http://pod1.example.org/a?")]
    #[case("http://pod1.example.org")]
    #[case("http://pod1.example.org/a?b/")]
    fn uri_sans_path_tslash_will_be_rejected(#[case] uri_str: &'static str) {
        let uri = HttpUri::try_from(uri_str).expect("Claimed valid uri str");
        assert_err_eq!(
            Proven::<_, PathHasTrailingSlash>::try_new(uri).map_err(|e| e.error),
            PathHasNoTrailingSlash
        );
    }

    #[rstest]
    #[case("http://pod1.example.org/a/")]
    #[case("http://pod1.example.org/")]
    #[case::with_fragment("http://pod1.example.org/a/#b/")]
    fn uri_with_path_tslash_will_be_accepted(#[case] uri_str: &'static str) {
        let uri = HttpUri::try_from(uri_str).expect("Claimed valid uri str");
        assert_ok!(Proven::<_, PathHasTrailingSlash>::try_new(uri),);
    }
}
