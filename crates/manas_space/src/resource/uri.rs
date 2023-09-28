//! I define types to represent resource uris in solid storage
//! space.
//!

use manas_http::uri::invariant::NormalAbsoluteHttpUri;

/// A resource uri is an http absolute uri in normal form.
pub type SolidResourceUri = NormalAbsoluteHttpUri;
