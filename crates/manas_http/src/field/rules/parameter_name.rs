//! I define `FieldParameterName` rule.

use std::{ops::Deref, str::FromStr};

use unicase::Ascii;

use super::token::{InvalidEncodedToken, Token};

/// A struct for representing field parameter_name as per [RFC 9110 Parameters]
///
/// > Parameter names are case-insensitive.
///
///  ```txt
///   parameter-name  = token
///  ```
///
///
/// [RFC 9110 Parameters]: https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldParameterName(Ascii<Token>);

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid field parameter name.
pub enum InvalidEncodedFieldParameterName {
    /// Case of invalid token.
    #[error("Given value is not a valid encoded token")]
    InvalidToken(#[from] InvalidEncodedToken),
}

impl FromStr for FieldParameterName {
    type Err = InvalidEncodedFieldParameterName;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token: Token = s.parse()?;
        Ok(Self(Ascii::new(token)))
    }
}

impl Deref for FieldParameterName {
    type Target = Ascii<Token>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FieldParameterName> for Ascii<Token> {
    #[inline]
    fn from(val: FieldParameterName) -> Self {
        val.0
    }
}

impl From<FieldParameterName> for Token {
    #[inline]
    fn from(val: FieldParameterName) -> Self {
        let ascii_token: Ascii<Token> = val.into();
        ascii_token.deref().clone()
    }
}

impl FieldParameterName {
    /// Push encoded str to given buffer
    #[inline]
    pub fn push_encoded_str(&self, buffer: &mut String) {
        buffer.push_str(self.as_ref());
    }

    /// Encode to str as per rfc
    #[inline]
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }

    #[inline]
    /// Get as token.
    pub fn as_token(&self) -> &Token {
        self.0.deref()
    }
}
