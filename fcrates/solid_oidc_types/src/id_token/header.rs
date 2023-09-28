//! I define type to represent solid-oidc-id-token header.
//!

use picky::jose::jws::JwsHeader;

/// A type to represent Jws header of solid-oidc-id-token.
pub type IdTokenHeader = JwsHeader;
