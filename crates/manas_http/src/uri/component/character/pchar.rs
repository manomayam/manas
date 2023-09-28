//! This module exports data regarding pchars, that are allowed in uri path segments as defined by [rfc3986#section-3.3](https://www.rfc-editor.org/rfc/rfc3986#section-3.3)
//!
//! From [rfc3986#section-3.3](https://www.rfc-editor.org/rfc/rfc3986#section-3.3):
//!
//! ```txt
//! pchar  = unreserved / pct-encoded / sub-delims / ":" / "@"
//! ```
//!

use percent_encoding::AsciiSet;

use super::{reserved::remove_uri_sub_delims, unreserved::UNRESERVED_PCT_ENCODE_SET};

/// A set of ascii chars, that are "not pchars", and therefore have to be pct-encoded. where pchar is defined as:
///
/// ```txt
/// pchar  = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
///
pub const PCHAR_PCT_ENCODE_SET: &AsciiSet = &remove_uri_sub_delims(UNRESERVED_PCT_ENCODE_SET)
    .remove(b':')
    .remove(b'@');

/// pct encode set, that is used to keep "/", and encode other non pchars.
pub const PATHCHAR_PCT_ENCODE_SET: &AsciiSet = &PCHAR_PCT_ENCODE_SET.remove(b'/');
