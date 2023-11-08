//! I define [`PFieldValue`] struct to represent
//! a parameterized field value.
//!

use ecow::EcoString;

use super::rules::{
    flat_csv::{split_field_params, SemiColon},
    parameters::FieldParameters,
};

/// A struct representing parameterized field value
#[derive(Debug, Clone)]
pub struct PFieldValue {
    /// Base value.
    pub base_value: EcoString,

    /// parameters.
    pub params: FieldParameters<SemiColon>,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid parameterized field value.
pub enum InvalidEncodedPFieldValue {
    /// Invalid field params.
    #[error("Invalid params in field value")]
    InvalidParams,
}

impl PFieldValue {
    /// Decode from raw field value
    pub fn decode(
        value: &str,
        allow_implicit_param_value: bool,
    ) -> Result<Self, InvalidEncodedPFieldValue> {
        let mut parts = split_field_params::<SemiColon>(value);
        let base_value = EcoString::from(
            parts
                .next()
                .expect("field value will have at least one csv part, even if empty"),
        );

        let params = FieldParameters::<SemiColon>::decode(parts, allow_implicit_param_value)
            .map_err(|_| InvalidEncodedPFieldValue::InvalidParams)?;
        Ok(Self { base_value, params })
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use rstest::rstest;

    use super::*;
    use crate::common::field::rules::parameters::tests_parse::assert_matches_param_records;

    #[rstest]
    #[case(
        r#"<http://example.com/TheBook/chapter2>; rel="previous"; title="previous chapter""#,
        r#"<http://example.com/TheBook/chapter2>"#,
        &[("rel", "previous"), ("title", "previous chapter")]
    )]
    #[case(
        r#"</>; rel="http://example.net/foo""#,
        r#"</>"#,
        &[("rel", "http://example.net/foo")]
    )]
    #[case(
        r##"</terms>; rel=copyright; anchor="#foo""##,
        "</terms>",
        &[("rel", "copyright"), ("anchor", "#foo")]
    )]
    #[case(
        r#"</TheBook/chapter2>; rel="previous"; title*=UTF-8'de'letztes%20Kapitel"#,
         "</TheBook/chapter2>",
         &[("rel", "previous"), ("title*", "UTF-8'de'letztes%20Kapitel")]
    )]
    #[case(
        r#"<http://example.org/>; rel="start http://example.net/relation/other""#,
         "<http://example.org/>",
         &[("rel", "start http://example.net/relation/other")]
    )]
    #[case::no_params("abc; ", "abc", &[])]
    #[case::no_params2("abc", "abc", &[])]
    #[case::no_params2("abc; rel=pqr;;", "abc", &[("rel", "pqr")])]
    #[case::empty("", "", &[])]
    fn valid_pfield_values_will_be_parsed_correctly(
        #[case] field_value_str: &str,
        #[case] expected_base_value: &str,
        #[case] expected_params: &[(&str, &str)],
    ) {
        let p_field_value = assert_ok!(PFieldValue::decode(field_value_str, false));

        assert_eq!(
            p_field_value.base_value, expected_base_value,
            "Base value parsed in correctly"
        );
        assert_matches_param_records(&p_field_value.params, expected_params);
    }

    #[rstest]
    #[case::no_kv("abc; pqr")]
    #[case::invalid_k("abc; a,b=cd")]
    #[case::unquoted_invalid("abc; a = b c")]
    #[case::misquoted("abc; a=\"bc")]
    fn invalid_pfield_values_will_be_rejected(#[case] field_value_str: &str) {
        assert_err!(PFieldValue::decode(field_value_str, false));
    }
}
