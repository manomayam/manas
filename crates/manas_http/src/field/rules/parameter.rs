//! I define [`FieldParameter`] struct corresponding to `parameter` rule.
//!
use std::str::FromStr;

use super::{
    parameter_name::{FieldParameterName, InvalidEncodedFieldParameterName},
    parameter_value::{FieldParameterValue, InvalidEncodedFieldParameterValue},
};

/// A struct for representing field parameter as per [RFC 9110 Parameters]
///
/// > Note: Parameters do not allow whitespace (not even "bad" whitespace) around the "=" character.
///
///  ```txt
///   parameter       = parameter-name "=" parameter-value
///  ```
///
/// [RFC 9110 Parameters]: https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldParameter {
    /// Parameter name.
    pub name: FieldParameterName,

    /// Parameter value.
    pub value: FieldParameterValue,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid field parameter.
pub enum InvalidEncodedFieldParameter {
    /// Invalid parameter encoding.
    #[error("Invalid parameter encoding")]
    InvalidParameterEncoding,

    /// Invalid parameter name.
    #[error("Invalid parameter name")]
    InvalidParameterName(#[from] InvalidEncodedFieldParameterName),

    /// Invalid parameter value.
    #[error("Invalid parameter value")]
    InvalidParameterValue(#[from] InvalidEncodedFieldParameterValue),
}

impl FieldParameter {
    /// Decode header param from encoded string.
    /// It follows following abnf
    /// ```txt
    /// parameter = token BWS "=" BWS ( token / quoted-string )
    /// ```
    ///
    /// If `allow_implicit_param_value` is true, then parameter value can be omitted for empty value like:
    ///
    /// ```txt
    /// parameter = token BWS
    /// ```
    pub fn decode(
        value: &str,
        allow_implicit_param_value: bool,
    ) -> Result<Self, InvalidEncodedFieldParameter> {
        let mut value_split = value.split_once('=');
        if value_split.is_none() && allow_implicit_param_value {
            value_split = Some((value, "\"\""));
        }
        let (k, v) = value_split.ok_or(InvalidEncodedFieldParameter::InvalidParameterEncoding)?;

        Ok(Self {
            name: FieldParameterName::from_str(k.trim())?,
            value: FieldParameterValue::decode(v.trim())?,
        })
    }

    /// Push encoded str to given buffer
    #[inline]
    pub fn push_encoded_str(&self, buffer: &mut String) {
        self.name.push_encoded_str(buffer);
        buffer.push('=');
        self.value.push_encoded_str(buffer);
    }

    /// Encode to str as per rfc
    #[inline]
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }
}

#[cfg(test)]
mod tests_decode {
    use claims::{assert_err, assert_err_eq, assert_matches, assert_ok};
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("abc")]
    #[case("abc def")]
    fn invalid_encoded_param_will_be_rejected(#[case] param_str: &str) {
        assert_err_eq!(
            FieldParameter::decode(param_str, false),
            InvalidEncodedFieldParameter::InvalidParameterEncoding
        );
    }

    #[rstest]
    #[case("a/b = cd")]
    #[case("a{b} = cd")]
    #[case("a b = cd")]
    #[case("रा = _")]
    fn param_with_invalid_key_will_be_rejected(#[case] param_str: &str) {
        assert_matches!(
            claims::assert_err!(FieldParameter::decode(param_str, false)),
            InvalidEncodedFieldParameter::InvalidParameterName(..)
        );
    }

    #[rstest]
    #[case("ab = c/d")]
    #[case("ab = c{d}")]
    #[case("ab = \"ab\\\"c\"")]
    #[case("_ = रा")]
    #[case("_ = \"रा\"")]
    fn param_with_invalid_value_will_be_rejected(#[case] param_str: &str) {
        assert_matches!(
            assert_err!(FieldParameter::decode(param_str, false)),
            InvalidEncodedFieldParameter::InvalidParameterValue(..)
        );
    }

    fn assert_param_match(
        param: &FieldParameter,
        expected_key_str: &str,
        expected_value_str: &str,
    ) {
        assert_eq!(
            param.name,
            FieldParameterName::from_str(expected_key_str).unwrap()
        );
        assert_eq!(param.value.as_ref(), expected_value_str);
    }

    #[rstest]
    #[case("a=b", "a", "b")]
    #[case("a = b", "a", "b")]
    #[case("abc=\"b\"", "abc", "b")]
    #[case("KK=b", "kk", "b")]
    #[case("a=b", "a", "b")]
    #[case("a=\"b c d e\"", "a", "b c d e")]
    #[case("a=b.png", "a", "b.png")]
    fn valid_param_will_be_parsed_correctly(
        #[case] param_str: &str,
        #[case] expected_key_str: &str,
        #[case] expected_value_str: &str,
    ) {
        let param = assert_ok!(FieldParameter::decode(param_str, false));
        assert_param_match(&param, expected_key_str, expected_value_str);
    }

    #[rstest]
    #[case("a", "a", "")]
    #[case("a = b", "a", "b")]
    #[case("abc=\"\"", "abc", "")]
    #[case("KK", "kk", "")]
    fn valid_implicit_param_will_be_parsed_correctly(
        #[case] param_str: &str,
        #[case] expected_key_str: &str,
        #[case] expected_value_str: &str,
    ) {
        let param = assert_ok!(FieldParameter::decode(param_str, true));
        assert_param_match(&param, expected_key_str, expected_value_str);
    }
}
