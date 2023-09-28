//! This module exports data regarding reserved characters in a URI as defined by [rfc3986#section-2.2](https://www.rfc-editor.org/rfc/rfc3986#section-2.2)
//!
//! From [rfc3986#section-2.2](https://www.rfc-editor.org/rfc/rfc3986#section-2.2):
//!
//! ```txt
//! reserved    = gen-delims / sub-delims
//! gen-delims  = ":" / "/" / "?" / "#" / "[" / "]" / "@"
//! sub-delims  = "!" / "$" / "&" / "'" / "(" / ")"
//!                  / "*" / "+" / "," / ";" / "="
//! ```

use percent_encoding::AsciiSet;

/// Adds URI gen-delims to given ascii-set
pub const fn add_uri_gen_delims(ascii_set: &AsciiSet) -> AsciiSet {
    ascii_set
        .add(b':')
        .add(b'/')
        .add(b'?')
        .add(b'#')
        .add(b'[')
        .add(b']')
        .add(b'@')
}

/// Adds URI sub-delims to given ascii-set
pub const fn add_uri_sub_delims(ascii_set: &AsciiSet) -> AsciiSet {
    ascii_set
        .add(b'!')
        .add(b'$')
        .add(b'&')
        .add(b'\'')
        .add(b'(')
        .add(b')')
        .add(b'*')
        .add(b'+')
        .add(b',')
        .add(b';')
        .add(b'=')
}

/// Removes URI sub-delims to given ascii-set
pub const fn remove_uri_sub_delims(ascii_set: &AsciiSet) -> AsciiSet {
    ascii_set
        .remove(b'!')
        .remove(b'$')
        .remove(b'&')
        .remove(b'\'')
        .remove(b'(')
        .remove(b')')
        .remove(b'*')
        .remove(b'+')
        .remove(b',')
        .remove(b';')
        .remove(b'=')
}

/// Adds URI reserved characters to given ascii-set
pub const fn add_uri_reserved_chars(ascii_set: &AsciiSet) -> AsciiSet {
    add_uri_gen_delims(&add_uri_sub_delims(ascii_set))
}
