//! I define handle and description types for acp access control resources.
//!

use rdf_utils::define_handle_and_description_types;
use rdf_vocabularies::ns;

use super::{
    access_control::{DAccessControl, HAccessControl},
    resource::HResource,
};

define_handle_and_description_types!(
    /// A handle type for [acp access control resources](https://solid.github.io/authorization-panel/acp-specification/#access-control-resource).
    ///
    /// > Instances of the Access Control Resource (ACR) class connect resources to their Access Controls.
    ///
    HAccessControlResource;
    /// A type alias for acp access control resource description.
    DAccessControlResource;

    [
        /// > The resource property connects ACRs to resources they control.
        /// > It is the inverse of acp:accessControlResource.
        (resource, &ns::acp::resource, HResource);

        /// > The access control property connects ACRs to Access Controls.
        (access_control, &ns::acp::accessControl, HAccessControl, DAccessControl);

        /// > The member access control property transitively connects ACRs of member resources to Access Controls.
        (member_access_control, &ns::acp::memberAccessControl, HAccessControl, DAccessControl);
    ]
);
