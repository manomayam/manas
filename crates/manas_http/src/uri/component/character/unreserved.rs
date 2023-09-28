//! This module exports data regarding unreserved characters in a URI as defined by [rfc3986#section-2.3](https://www.rfc-editor.org/rfc/rfc3986#section-2.3)
//!
//! From [rfc3986#section-2.3](https://www.rfc-editor.org/rfc/rfc3986#section-2.3):
//!
//! ```txt
//! Characters that are allowed in a URI but do not have a reserved
//! purpose are called unreserved.  These include uppercase and lowercase
//! letters, decimal digits, hyphen, period, underscore, and tilde.
//!
//!     unreserved  = ALPHA / DIGIT / "-" / "." / "_" / "~"
//! ```
//!

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};

/// A set of ascii chars, that are "not unreserved", and therefore have to be pct-encoded. where unreserved is defined as:
///
/// ```txt
///      unreserved  = ALPHA / DIGIT / "-" / "." / "_" / "~"
/// ```
///
pub const UNRESERVED_PCT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');
