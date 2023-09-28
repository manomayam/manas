//! I define few static aux rel types, and mock helpers.

pub mod known;

use manas_http::define_static_rel_types;

define_static_rel_types!(
    /// Acl rel type.
    ACL_REL_TYPE: "acl";

    /// Described by rel type.
    DESCRIBED_BY_REL_TYPE: "describedby";

    /// Container index rel type.
    CONTAINER_INDEX_REL_TYPE: "containerindex";
);

/// I define utilities for easily mocking aux rel types.
#[cfg(feature = "test-utils")]
pub mod mock {
    use manas_http::define_static_rel_types;

    pub use super::{ACL_REL_TYPE, CONTAINER_INDEX_REL_TYPE, DESCRIBED_BY_REL_TYPE};

    // Define few mock rel types.
    define_static_rel_types!(
        /// Mock rel type "ta1"
        TA1_REL_TYPE: "ta1";
        /// Mock rel type "ta2"
        TA2_REL_TYPE: "ta2";
        /// Mock rel type "tc1"
        TC1_REL_TYPE: "tc1";
        /// Mock rel type "tc2"
        TC2_REL_TYPE: "tc2";
        /// Mock rel type "tn1"
        TN1_REL_TYPE: "tn1";
        /// Mock rel type "tn2"
        TN2_REL_TYPE: "tn2";
    );
}
