//! I define  a handle, description types for acp access grants.
//!

use rdf_utils::define_handle_and_description_types;
use rdf_vocabularies::ns;

use super::{
    access_mode::HAccessMode,
    context::{DContext, HContext},
};

define_handle_and_description_types!(
    /// A struct to represent [acp access grants](https://solid.github.io/authorization-panel/acp-specification/#access-grant).
    ///
    /// > Instances of the Access Grant class define sets of Access Modes granted in particular Contexts.
    ///
    HAccessGrant;
    /// A type alias for acp access grant description.
    DAccessGrant;

    [
        /// > The context property connects Access Grants to the Contexts in which they're given.
        (context, &ns::acp::context, HContext, DContext);

        /// > The grant property connects Access Grants to the Access Modes they grant.
        (grant, &ns::acp::grant, HAccessMode);
    ]
);
