//! I define [`FieldParameterValue`.] struct.
//!
//!
use std::{ops::Deref, str::FromStr};

use ecow::EcoString;

use super::token::Token;

/// A struct for representing field parameter_value as per [RFC 9110 Parameters]
/// > Parameter values might or might not be case-sensitive,.
/// > A parameter value that matches the token production can be transmitted either as a token or within a quoted-string.
/// > The quoted and unquoted values are equivalent.
///
///  ```txt
///  parameter-value = ( token / quoted-string )
///  ```
///
///
/// [RFC 9110 Parameters]: https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldParameterValue(EcoString);

/// Constant for double quote.
pub(crate) const DQUOTE: &str = "\"";

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid parameter value.
pub enum InvalidEncodedFieldParameterValue {
    /// Non Ascii text.
    #[error("Given value is non ascii")]
    NonAsciiText,

    /// Invalid double quotes.
    #[error("Given value contains invalid double quotes")]
    InvalidDoubleQuotes,

    /// Invalid token.
    #[error("Given unquoted value is not a valid token")]
    InvalidToken,
}

impl Deref for FieldParameterValue {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for FieldParameterValue {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for FieldParameterValue {
    type Error = InvalidEncodedFieldParameterValue;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if !s.is_ascii() {
            return Err(Self::Error::NonAsciiText);
        }
        if s.contains(DQUOTE) {
            return Err(Self::Error::InvalidDoubleQuotes);
        }
        Ok(Self(s.into()))
    }
}

impl FieldParameterValue {
    /// Decodes param value from optionally quoted string or unquoted token
    pub fn decode(value: &str) -> Result<Self, InvalidEncodedFieldParameterValue> {
        if !value.is_ascii() {
            return Err(InvalidEncodedFieldParameterValue::NonAsciiText);
        }
        // If value is quoted string
        if value.starts_with(DQUOTE) && value.ends_with(DQUOTE) {
            let value_unquoted = &value[1..value.len() - 1];
            if value_unquoted.contains(DQUOTE) {
                return Err(InvalidEncodedFieldParameterValue::InvalidDoubleQuotes);
            }
            Ok(Self(value_unquoted.into()))
        }
        // Else value is token
        else {
            Ok(Self(
                Token::from_str(value)
                    .map_err(|_| InvalidEncodedFieldParameterValue::InvalidToken)?
                    .into(),
            ))
        }
    }

    /// Push encoded value to buffer
    #[inline]
    pub fn push_encoded_str(&self, buffer: &mut String) {
        // TODO use token value form when possible
        buffer.push_str(DQUOTE);
        buffer.push_str(self.0.as_str());
        buffer.push_str(DQUOTE);
    }

    /// Get encoded value
    #[inline]
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }
}

#[cfg(test)]
mod tests_decode {
    use claims::{assert_err_eq, assert_ok};
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("ab cd ef")]
    #[case("ab/cd")]
    #[case("ab,cd")]
    #[case::empty("")]
    fn invalid_unquoted_str_will_be_rejected(#[case] value: &str) {
        assert_err_eq!(
            FieldParameterValue::decode(value),
            InvalidEncodedFieldParameterValue::InvalidToken
        );
    }

    #[rstest]
    #[case(r#""abc"def""#)]
    #[case(r#""abcdef"""#)]
    #[case(r#"""def""#)]
    fn invalid_quoted_str_will_be_rejected(#[case] value: &str) {
        assert_err_eq!(
            FieldParameterValue::decode(value),
            InvalidEncodedFieldParameterValue::InvalidDoubleQuotes
        );
    }

    #[rstest]
    #[case::token1("abc", "abc")]
    #[case::token2("pqr.def", "pqr.def")]
    #[case::quoted1(r#""abc""#, r#"abc"#)]
    #[case::quoted1(r#""a/b/c""#, r#"a/b/c"#)]
    #[case::quoted1(r#""a b c ""#, r#"a b c "#)]
    fn valid_value_will_be_decoded_correctly(#[case] value: &str, #[case] expected: &str) {
        let param_value = assert_ok!(FieldParameterValue::decode(value));
        assert_eq!(param_value.as_ref(), expected);
    }
}
