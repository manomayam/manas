//! I define  a handle, description types for acp access modes.
//!

use rdf_utils::define_handle_type;
use rdf_vocabularies::ns;
use sophia_api::ns::NsTerm;

define_handle_type!(
    /// A handle type for [acp access modes](https://solid.github.io/authorization-panel/acp-specification/#access-mode-extensibility).
    ///
    /// > Instances of the Access Control Resource (ACR) class connect resources to their Access Controls.
    ///
    HAccessMode;
    []
);

/// Handle to `acl:Read` access mode.
pub static H_READ: HAccessMode<NsTerm> = unsafe { HAccessMode::new_unchecked(ns::acl::Read) };

/// Handle to `acl:Append` access mode.
pub static H_APPEND: HAccessMode<NsTerm> = unsafe { HAccessMode::new_unchecked(ns::acl::Append) };

/// Handle to `acl:Write` access mode.
pub static H_WRITE: HAccessMode<NsTerm> = unsafe { HAccessMode::new_unchecked(ns::acl::Write) };

/// Handle to `acl:Control` access mode.
pub static H_CONTROL: HAccessMode<NsTerm> = unsafe { HAccessMode::new_unchecked(ns::acl::Control) };
