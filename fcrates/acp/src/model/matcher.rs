//! I define handle and description types for acp matchers.
//!

use rdf_utils::define_handle_and_description_types;

define_handle_and_description_types!(
    /// A struct handle type for [acp matchers](https://solid.github.io/authorization-panel/acp-specification/#matcher).
    ///
    /// > Instances of the Matcher class are descriptions of matching Contexts.
    ///
    HMatcher;
    /// A type alias for acp matcher description.
    DMatcher;
    []
);
