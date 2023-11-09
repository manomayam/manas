//! I define [`FieldParameter`s] struct corresponding to `* parameter` rule.
//!
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::{
    flat_csv::{SemiColon, Separator},
    parameter::{FieldParameter, InvalidEncodedFieldParameter},
    parameter_name::FieldParameterName,
    parameter_value::FieldParameterValue,
};

/// A struct for representing field parameters as per [RFC 9110 Parameters]
///
/// > Parameters are instances of name/value pairs;
/// > they are often used in field values as a common syntax for appending auxiliary information to an item.
/// > Each parameter is usually delimited by an immediately preceding semicolon.
///
/// > Parameter names are case-insensitive.
///
///  ```txt
///   parameters      = *( OWS ";" OWS [ parameter ] )
///   parameter       = parameter-name "=" parameter-value
///   parameter-name  = token
///   parameter-value = ( token / quoted-string )
///  ```
///
/// [RFC 9110 Parameters]: https://www.rfc-editor.org/rfc/rfc9110.html#section-5.6.6
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldParameters<Sep = SemiColon> {
    /// List of params.
    pub params: Vec<FieldParameter>,
    _phantom: PhantomData<fn(Sep)>,
}

impl<Sep> FieldParameters<Sep> {
    /// Create a new [`FieldParameters`] with no items.
    #[inline]
    pub fn new(params: Vec<FieldParameter>) -> Self {
        Self {
            params,
            _phantom: PhantomData,
        }
    }
}

impl<Sep> Deref for FieldParameters<Sep> {
    type Target = Vec<FieldParameter>;

    fn deref(&self) -> &Self::Target {
        &self.params
    }
}

impl DerefMut for FieldParameters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.params
    }
}

#[derive(Debug, Clone, thiserror::Error)]
/// Error of onvalid field parameters.
pub enum InvalidEncodedFieldParameters {
    /// Invalid parameter.
    #[error("Invalid parameter")]
    InvalidParameter(#[from] InvalidEncodedFieldParameter),
}

impl<Sep: Separator> FieldParameters<Sep> {
    /// Try decode new header value params from given iterator of param strings
    pub fn decode(
        param_strs: impl Iterator<Item = impl AsRef<str>>,
        allow_implicit_param_value: bool,
    ) -> Result<Self, InvalidEncodedFieldParameters> {
        Ok(Self::new(
            param_strs
                .filter(|s| !s.as_ref().is_empty())
                .map(|s| FieldParameter::decode(s.as_ref(), allow_implicit_param_value))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    /// Pushes encoded params to given string buffer
    pub fn push_encoded_str(&self, buffer: &mut String) {
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                buffer.push(Sep::CHAR);
                buffer.push(' ');
            }
            buffer.push_str(param.name.as_ref());
            buffer.push('=');
            param.value.push_encoded_str(buffer);
        }
    }

    /// Encode params as string
    #[inline]
    pub fn encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }

    /// Get value of the first matching parameter with given param name.
    pub fn get_value(&self, name: &FieldParameterName) -> Option<&FieldParameterValue> {
        self.iter().find_map(|p| {
            if &p.name == name {
                Some(&p.value)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
pub(crate) mod tests_parse {
    use claims::{assert_err, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        &[r#"rel="previous""#, r#"title="previous chapter""#],
        &[("rel", "previous"), ("title", "previous chapter")]
    )]
    #[case(
        &[r#"rel="http://example.net/foo""#],
        &[("rel", "http://example.net/foo")]
    )]
    #[case(
        &[r#"rel=copyright"#, r##"anchor="#foo""##],
        &[("rel", "copyright"), ("anchor", "#foo")]
    )]
    #[case(
        &[r#"rel="previous""#, r#"title*=UTF-8'de'letztes%20Kapitel"#],
        &[("rel", "previous"), ("title*", "UTF-8'de'letztes%20Kapitel")]
    )]
    #[case(
        &[r#"rel="start http://example.net/relation/other""#, ],
        &[("rel", "start http://example.net/relation/other")]
    )]
    fn valid_params_will_be_parsed_correctly(
        #[case] params_list: &[&str],
        #[case] expected_param_records: &[(&str, &str)],
    ) {
        let params = assert_ok!(FieldParameters::decode(params_list.iter(), false));
        assert_matches_param_records(&params, expected_param_records);
    }

    pub fn assert_matches_param_records(params: &FieldParameters, param_records: &[(&str, &str)]) {
        assert_eq!(params.len(), param_records.len());
        assert!(
            params
                .iter()
                .zip(param_records.iter())
                .all(|(param, (n, v))| param.name.eq_ignore_ascii_case(n)
                    && param.value.as_ref().eq(*v)),
            "Params parsed incorrectly"
        );
    }

    #[rstest]
    #[case(&["abc", "pqr"])]
    #[case(&["abc=def", "pqr"])]
    #[case(&["abc=", "pqr"])]
    #[case(&["abc=rd", "pqr=c a b"])]
    #[case(&[r#"rel="previous""#, r#"title*=UTF-8'de'letztes%20Kapitel""#],)]
    fn invalid_params_will_be_rejected(#[case] params_list: &[&str]) {
        assert_err!(FieldParameters::<SemiColon>::decode(
            params_list.iter(),
            false
        ));
    }
}

#[cfg(test)]
mod tests_encode {
    use claims::assert_ok;
    use rstest::rstest;

    use super::{tests_parse::*, *};
    use crate::common::field::rules::flat_csv::{FlatCsv, SemiColon};

    #[rstest]
    #[case(
        &[r#"rel="previous""#, r#"title="previous chapter""#],
        &[("rel", "previous"), ("title", "previous chapter")]
    )]
    #[case(
        &[r#"rel="http://example.net/foo""#],
        &[("rel", "http://example.net/foo")]
    )]
    #[case(
        &[r#"rel=copyright"#, r##"anchor="#foo""##],
        &[("rel", "copyright"), ("anchor", "#foo")]
    )]
    #[case(
        &[r#"rel="previous""#, r#"title*=UTF-8'de'letztes%20Kapitel"#],
        &[("rel", "previous"), ("title*", "UTF-8'de'letztes%20Kapitel")]
    )]
    #[case(
        &[r#"rel="start http://example.net/relation/other""#, ],
        &[("rel", "start http://example.net/relation/other")]
    )]
    fn round_trip_works_correctly(
        #[case] params_list: &[&str],
        #[case] expected_param_records: &[(&str, &str)],
    ) {
        let params = assert_ok!(FieldParameters::<SemiColon>::decode(
            params_list.iter(),
            false
        ));
        let params_encoded = params.encode();

        let params_round_tripped = assert_ok!(FieldParameters::decode(
            FlatCsv::<SemiColon>::from(&assert_ok!(headers::HeaderValue::from_str(
                &params_encoded
            )))
            .iter(),
            false
        ));

        assert_matches_param_records(&params_round_tripped, expected_param_records);
    }
}
