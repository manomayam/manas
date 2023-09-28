//! I define [`ProblemType`] struct, that defines a problem type.
//!

use http::StatusCode;
use http_api_problem::{ApiError, ApiErrorBuilder};
use iri_string::types::UriReferenceString;

use super::{Problem, ProblemBuilder};

/// Description of a problem type based on [RFC7807](https://tools.ietf.org/html/rfc7807).
pub struct ProblemType {
    /// Id of problem.
    /// A URI reference [RFC3986](https://tools.ietf.org/html/rfc3986) that identifies the
    /// problem type.  This specification encourages that, when
    /// dereferenced, it provide human-readable documentation for the
    /// problem type (e.g., using HTML [W3C.REC-html5-20141028]).  
    pub id: UriReferenceString,

    ///A short, human-readable summary of the problem type
    pub title: String,
}

/// Type alias for lazy static problem type.
pub type LazyStaticProblemType = once_cell::sync::Lazy<ProblemType>;

impl ProblemType {
    /// Create a new [`ProblemType`] with an anonymous id and given title.
    #[cfg(feature = "anon-problem-type")]
    pub fn new_anonymous(title: impl Into<String>) -> Self {
        Self {
            id: format!("urn::dyn_problem::_anon/{}", uuid::Uuid::new_v4())
                .parse()
                .expect("Must be a valid urm."),
            title: title.into(),
        }
    }

    /// Get new [`Problem`] with this problem type.
    #[inline]
    pub fn new_problem(&self) -> Problem {
        self.new_problem_builder().finish()
    }

    /// Get a new problem builder with this problem type.
    #[inline]
    pub fn new_problem_builder(&self) -> ProblemBuilder {
        Problem::builder()
            .type_url(self.id.to_string())
            .title(self.title.clone())
    }

    /// Get new [`ApiErrorBuilder`] with this problem type.
    #[inline]
    pub fn new_api_error_builder(&self, status: StatusCode) -> ApiErrorBuilder {
        ApiError::builder(status)
            .type_url(self.id.to_string())
            .title(self.title.clone())
    }

    /// Check if this problem type is the type of given problem.
    #[inline]
    pub fn is_type_of(&self, e: &Problem) -> bool {
        e.type_url()
            .map(|type_url| self.id.as_str() == type_url)
            .unwrap_or(false)
    }

    /// Check if this problem type is the type of given api error.
    #[inline]
    pub fn is_type_of_api_err(&self, e: &ApiError) -> bool {
        e.type_url()
            .map(|type_url| self.id.as_str() == type_url)
            .unwrap_or(false)
    }
}

/// Macro to easily define problem types in a namespace.
#[macro_export(local_inner_macros)]
#[cfg(feature = "anon-problem-type")]
macro_rules! define_anon_problem_types {
    (
        $($(#[$outer:meta])*$PROBLEM:ident: ($title:expr);)*
    ) => {
        $(
            #[allow(missing_docs)]
            $(#[$outer])*
            pub static $PROBLEM: $crate::type_::LazyStaticProblemType = $crate::type_::LazyStaticProblemType::new(|| {
                    $crate::type_::ProblemType::new_anonymous($title)
            });
        )*
    };
}

define_anon_problem_types!(
    /// An internal error.
    INTERNAL_ERROR: ("Internal error");

    /// Unknown io error.
    UNKNOWN_IO_ERROR: (
        "Unknown io error."
    );

    /// Infallible.
    INFALLIBLE: ("Infallible.");
);
