//! I define few invariants of [`WebId`].
//!

use gdp_rs::Proven;
use http_uri::predicate::is_secure::IsSecure;

use crate::WebId;

/// Type alias for an invariant of [`WebId`] that is proven to be secure as per given secure transport policy.
pub type SecureWebId<STP> = Proven<WebId, IsSecure<STP>>;
