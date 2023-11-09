//! I define [`AppliedPref`] struct.
//! corresponding to `applied-pref` production.
use std::str::FromStr;

use crate::{
    common::field::rules::{
        flat_csv::SemiColon, parameter_name::FieldParameterName,
        parameter_value::FieldParameterValue, parameters::FieldParameters,
    },
    prefer::{InvalidEncodedPreference, Preference},
};

/// Representation of `applied-pref` production as defined in [rfc7240`](https://datatracker.ietf.org/doc/html/rfc7240#section-3)
///
/// ABNF:
/// ````txt
/// applied-pref = token [ BWS "=" BWS word ]
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedPref(Preference);

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid applied pref.
pub enum InvalidEncodedAppliedPref {
    /// Invalid preference.
    #[error("Invalid preference")]
    InvalidPreference(#[from] InvalidEncodedPreference),

    /// Invalid extra params.
    #[error("applied-pref must not have any params")]
    InvalidExtraParams,
}

impl FromStr for AppliedPref {
    type Err = InvalidEncodedAppliedPref;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let preference = Preference::from_str(value)?;
        if preference.params.len() != 0 {
            return Err(InvalidEncodedAppliedPref::InvalidExtraParams);
        }
        Ok(Self(preference))
    }
}

impl AppliedPref {
    /// Create new applied pref
    pub fn new(token_name: FieldParameterName, token_value: FieldParameterValue) -> Self {
        Self(Preference {
            token_name,
            token_value,
            params: FieldParameters::<SemiColon>::new(Vec::new()),
        })
    }

    // /// Push encoded value to buffer
    // #[inline]
    // pub(crate) fn push_encoded_str(&self, buffer: &mut String) {
    //     self.0.push_encoded_str(buffer)
    // }

    /// Get encoded value as per rfc
    #[inline]
    pub fn str_encode(&self) -> String {
        self.0.str_encode()
    }

    /// Get applied pref token name
    #[inline]
    pub fn token_name(&self) -> &FieldParameterName {
        &self.0.token_name
    }

    /// Get applied pref token value
    #[inline]
    pub fn token_value(&self) -> &FieldParameterValue {
        &self.0.token_value
    }
}

#[cfg(test)]
pub(crate) mod tests_decode {
    use claims::*;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("abc; def = 123")]
    #[case("abc; def=ab")]
    fn applied_pref_with_invalid_params_will_be_rejected(#[case] applied_pref_str: &str) {
        assert_matches!(
            assert_err!(AppliedPref::from_str(applied_pref_str)),
            InvalidEncodedAppliedPref::InvalidExtraParams
        );
    }

    #[rstest]
    #[case("abc/def")]
    #[case("abc def")]
    #[case("abc = def/pqr")]
    #[case("abc = def pqr")]
    fn applied_pref_with_invalid_applied_pref_token_will_be_rejected(
        #[case] applied_pref_str: &str,
    ) {
        assert_matches!(
            assert_err!(AppliedPref::from_str(applied_pref_str)),
            InvalidEncodedAppliedPref::InvalidPreference(..)
        );
    }

    pub fn assert_valid_applied_pref(applied_pref_str: &str) -> AppliedPref {
        assert_ok!(AppliedPref::from_str(applied_pref_str))
    }

    pub fn assert_applied_pref_match(
        applied_pref: &AppliedPref,
        expected_token_name: &str,
        expected_token_value: &str,
    ) {
        assert_eq!(
            applied_pref.token_name(),
            &expected_token_name.parse().unwrap()
        );
        assert_eq!(applied_pref.token_value().as_ref(), expected_token_value);
    }

    #[rstest]
    #[case("foo", "foo", "")]
    #[case("foo=\"\"", "foo", "")]
    #[case("respond-async", "respond-async", "")]
    #[case("return=minimal", "return", "minimal")]
    #[case(r#"return=representation"#, "return", "representation")]
    fn valid_applied_pref_will_be_round_tripped_correctly(
        #[case] applied_pref_str: &str,
        #[case] expected_token_name: &str,
        #[case] expected_token_value: &str,
    ) {
        let preference = assert_valid_applied_pref(applied_pref_str);
        assert_applied_pref_match(&preference, expected_token_name, expected_token_value);

        let encoded_applied_pref_str = preference.str_encode();
        let round_tripped_preference = assert_valid_applied_pref(&encoded_applied_pref_str);
        assert_applied_pref_match(
            &round_tripped_preference,
            expected_token_name,
            expected_token_value,
        );
    }
}
