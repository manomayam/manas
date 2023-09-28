//! I define handle and description types for acp policies.
//!

use rdf_utils::define_handle_and_description_types;
use rdf_vocabularies::ns;

use super::{
    access_mode::HAccessMode,
    matcher::{DMatcher, HMatcher},
};

define_handle_and_description_types!(
    /// A handle type for [acp policies](https://solid.github.io/authorization-panel/acp-specification/#policy).
    ///
    /// > Instances of the Policy class connect Access Controls to allowed and denied Access Modes as well as sets of matchers describing instances of resource access.
    ///
    HPolicy;
    /// A type alias for acp policy description.
    DPolicy;

    [
        /// > The allow property connects Policies to the Access Modes they allow if satisfied.
        (allow, &ns::acp::allow, HAccessMode);

        /// > The deny property connects Policies to the Access Modes they deny if satisfied.
        (deny, &ns::acp::deny, HAccessMode);

        /// > The all of property connects Policies to a set of matchers,
        /// > all of which MUST be satisfied for the Policy to be satisfied.
        (all_of, &ns::acp::allOf, HMatcher, DMatcher);

        /// > The all of property connects Policies to a set of matchers,
        /// > all of which MUST be satisfied for the Policy to be satisfied.
        (any_of, &ns::acp::anyOf, HMatcher, DMatcher);

        /// > The none of property connects Policies to a set of matchers,
        /// > all of which MUST NOT be satisfied for the Policy to be satisfied.
        (none_of, &ns::acp::noneOf, HMatcher, DMatcher);
    ]
);
