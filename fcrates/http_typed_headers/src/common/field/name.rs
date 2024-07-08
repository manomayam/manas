//! I define [`FieldName`] struct corresponding to `field-name` rule.
//!

use unicase::Ascii;

use super::rules::token::Token;

/// A struct for representing [RFC 9110 Field Names]
///
/// > A field name labels the corresponding field value as having the semantics defined by that name.
/// > Field names are case-insensitive.
///
///  ```txt
///   field-name     = token
///  ```
///
///
/// [RFC 9110 Field Names]: https://www.rfc-editor.org/rfc/rfc9110.html#name-field-names
pub struct FieldName(pub Ascii<Token>);
