//! I define [`Preference] struct
//! corresponding to `preference` production.
//!
use std::str::FromStr;

use once_cell::sync::Lazy;

use crate::field::{
    pvalue::{InvalidEncodedPFieldValue, PFieldValue},
    rules::{
        parameter::{FieldParameter, InvalidEncodedFieldParameter},
        parameter_name::FieldParameterName,
        parameter_value::FieldParameterValue,
        parameters::FieldParameters,
    },
};

/// A preference in a `Prefer` header as defined by rfc7240
///
/// It follows ABNF of `preference` production as in rfc:
/// ```txt
/// preference = token [ BWS "=" BWS word ]
///             *( OWS ";" [ OWS parameter ] )
/// parameter  = token [ BWS "=" BWS word ]
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preference {
    /// Token name.
    pub token_name: FieldParameterName,

    /// Token value.
    pub token_value: FieldParameterValue,

    /// parametets.
    pub params: FieldParameters,
}

/// Static for "return" token name.
pub static TOKEN_NAME_RETURN: Lazy<FieldParameterName> =
    Lazy::new(|| "return".parse().expect("Must be valid."));

/// Static for "include" param name.
pub static PARAM_NAME_INCLUDE: Lazy<FieldParameterName> =
    Lazy::new(|| "include".parse().expect("Must be valid."));

/// Static for "representation" token value.
pub static TOKEN_VALUE_REPRESENTATION: Lazy<FieldParameterValue> =
    Lazy::new(|| "representation".try_into().expect("Must be valid."));

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid `Preference`.
pub enum InvalidEncodedPreference {
    /// Invalid parameterized field value.
    #[error("Given header value is not a valid parameterized header value")]
    InvalidPFieldValue(#[from] InvalidEncodedPFieldValue),

    /// Invalid preference token.
    #[error("Given preference has invalid preference token")]
    InvalidPreferenceToken(#[from] InvalidEncodedFieldParameter),
}

impl FromStr for Preference {
    type Err = InvalidEncodedPreference;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let PFieldValue {
            base_value: pref_token,
            params,
        } = PFieldValue::decode(value, true)?;

        let FieldParameter {
            name: token_name,
            value: token_value,
        } = FieldParameter::decode(&pref_token, true)?;

        Ok(Self {
            token_name,
            token_value,
            params,
        })
    }
}

impl Preference {
    /// Push encoded value to buffer
    #[inline]
    pub(crate) fn push_encoded_str(&self, buffer: &mut String) {
        self.token_name.push_encoded_str(buffer);
        buffer.push('=');
        self.token_value.push_encoded_str(buffer);

        if self.params.len() > 0 {
            buffer.push_str("; ");
            self.params.push_encoded_str(buffer);
        }
    }

    /// Get encoded value as per rfc
    #[inline]
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }
}

#[cfg(test)]
pub(crate) mod tests_decode {
    use claims::*;
    use rstest::*;

    use super::*;
    use crate::field::rules::parameters::tests_parse::assert_matches_param_records;

    #[rstest]
    #[case::invalid_param_key("abc; def/ghi = 123")]
    #[case::invalid_param_value("abc; def=a b")]
    fn preference_with_invalid_params_will_be_rejected(#[case] preference_str: &str) {
        assert_matches!(
            assert_err!(Preference::from_str(preference_str)),
            InvalidEncodedPreference::InvalidPFieldValue(..)
        );
    }

    #[rstest]
    #[case("abc/def; a=1")]
    #[case("abc def; b=2")]
    #[case("abc = def/pqr; b=2")]
    #[case("abc = def pqr; b=2")]
    fn preference_with_invalid_preference_token_will_be_rejected(#[case] preference_str: &str) {
        assert_matches!(
            assert_err!(Preference::from_str(preference_str)),
            InvalidEncodedPreference::InvalidPreferenceToken(..)
        );
    }

    pub fn assert_valid_preference(preference_str: &str) -> Preference {
        assert_ok!(Preference::from_str(preference_str))
    }

    pub fn assert_preference_match(
        preference: &Preference,
        expected_token_name: &str,
        expected_token_value: &str,
        expected_params: &[(&str, &str)],
    ) {
        assert_eq!(preference.token_name, expected_token_name.parse().unwrap());
        assert_eq!(preference.token_value.as_ref(), expected_token_value);
        assert_matches_param_records(&preference.params, expected_params);
    }

    #[rstest]
    #[case("foo; bar", "foo", "", &[("bar","")])]
    #[case("foo; bar=\"\"", "foo", "", &[("bar","")])]
    #[case("foo=\"\"; bar", "foo", "", &[("bar","")])]
    #[case("respond-async; wait=100", "respond-async", "", &[("wait","100")])]
    #[case("return=minimal; foo=\"some parameter\"", "return", "minimal", &[("foo", "some parameter")])]
    #[case(
        r#"return=representation; include="http://www.w3.org/ns/ldp#PreferMembership http://www.w3.org/ns/ldp#PreferMinimalContainer""#,
        "return",
        "representation",
        &[("include", "http://www.w3.org/ns/ldp#PreferMembership http://www.w3.org/ns/ldp#PreferMinimalContainer")]
    )]
    fn valid_preference_will_be_round_tripped_correctly(
        #[case] preference_str: &str,
        #[case] expected_token_name: &str,
        #[case] expected_token_value: &str,
        #[case] expected_params: &[(&str, &str)],
    ) {
        let preference = assert_valid_preference(preference_str);
        assert_preference_match(
            &preference,
            expected_token_name,
            expected_token_value,
            expected_params,
        );

        let encoded_preference_str = preference.str_encode();
        let round_tripped_preference = assert_valid_preference(&encoded_preference_str);
        assert_preference_match(
            &round_tripped_preference,
            expected_token_name,
            expected_token_value,
            expected_params,
        );
    }
}
