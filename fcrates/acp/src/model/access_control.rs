//! I define  handle, description types for acp access controls.
//!

use rdf_utils::define_handle_and_description_types;
use rdf_vocabularies::ns;

use super::policy::{DPolicy, HPolicy};

define_handle_and_description_types!(
    /// A handle type for [acp access contros](https://solid.github.io/authorization-panel/acp-specification/#access-control).
    ///
    /// > Instances of the Access Control class connect Access Control Resources to their Policies.
    ///
    HAccessControl;
    /// A type alias for acp access control description.
    DAccessControl;

    [
        /// > The apply property connects Access Controls to the Policies they apply to resources.
        (apply, &ns::acp::apply, HPolicy, DPolicy);
    ]
);
