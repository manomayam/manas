//! I define [LinkValue] struct.
//!
use std::{ops::Deref, str::FromStr};

use headers::HeaderValue;
use iri_string::types::{UriReferenceStr, UriReferenceString};
use once_cell::sync::Lazy;

use super::{
    rel::{InvalidEncodedLinkRel, LinkRel, REL_PARAM_NAME},
    target::{InvalidEncodedLinkTarget, LinkTarget},
};
use crate::common::field::{
    pvalue::{InvalidEncodedPFieldValue, PFieldValue},
    rules::{
        flat_csv::SemiColon, parameter::FieldParameter, parameter_name::FieldParameterName,
        parameter_value::FieldParameterValue, parameters::FieldParameters,
    },
};

/// Constant for `anchor` parameter name.
pub static ANCHOR_PARAM_NAME: Lazy<FieldParameterName> =
    Lazy::new(|| "anchor".parse().expect("Must be valid"));

/// `LinkValue` is defined in [`rfc8288`](https://datatracker.ietf.org/doc/html/rfc8288#section-3)
///
/// ```txt
/// link-value = "<" URI-Reference ">" *( OWS ";" OWS link-param )
/// link-param = token BWS [ "=" BWS ( token / quoted-string ) ]
/// ```
#[derive(Debug, Clone)]
pub struct LinkValue<TargetUriRef = UriReferenceString> {
    target: LinkTarget<TargetUriRef>,
    params: FieldParameters<SemiColon>,
    rel: LinkRel,
    anchor: Option<UriReferenceString>,
}

#[derive(Debug, Clone, thiserror::Error)]
/// Error of invalid link value.
pub enum InvalidEncodedLinkValue {
    /// Invalid parameterized field value.
    #[error("Given header value is not a valid parameterized header value")]
    InvalidPFieldValue(#[from] InvalidEncodedPFieldValue),

    /// Invalid target.
    #[error("Invalid link target")]
    InvalidLinkTarget(#[from] InvalidEncodedLinkTarget),

    /// No `rel` param.
    #[error("Given link value has no rel param")]
    NoRelParam,

    /// Invalid `rel` param.
    #[error("Invalid rel param")]
    InvalidRelParam(#[from] InvalidEncodedLinkRel),

    /// Invalid `anchor` param.
    #[error("Invalid anchor param")]
    InvalidAnchorParam(#[from] iri_string::validate::Error),
}

impl<TargetUriRef> FromStr for LinkValue<TargetUriRef>
where
    TargetUriRef: for<'a> TryFrom<&'a str>,
{
    type Err = InvalidEncodedLinkValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let PFieldValue {
            base_value: encoded_target_str,
            params,
        } = PFieldValue::decode(s, false)?;

        let target = LinkTarget::decode(encoded_target_str.as_str())?;
        let rel = LinkRel::decode(
            params
                .get_value(REL_PARAM_NAME.deref())
                .map(|v| v.deref())
                .ok_or(InvalidEncodedLinkValue::NoRelParam)?,
        )?;

        let anchor = params
            .get_value(ANCHOR_PARAM_NAME.deref())
            .map(|v| UriReferenceString::try_from(v.as_ref()))
            .transpose()?;

        Ok(Self {
            target,
            params,
            rel,
            anchor,
        })
    }
}

impl<TargetUriRef> From<&LinkValue<TargetUriRef>> for HeaderValue
where
    TargetUriRef: AsRef<UriReferenceStr>,
{
    #[inline]
    fn from(val: &LinkValue<TargetUriRef>) -> Self {
        HeaderValue::from_str(val.str_encode().as_str()).expect("It must be valid header value")
    }
}

impl<TargetUriRef> LinkValue<TargetUriRef> {
    /// Get target of this link
    #[inline]
    pub fn target(&self) -> &LinkTarget<TargetUriRef> {
        &self.target
    }

    /// Get rel of this link
    #[inline]
    pub fn rel(&self) -> &LinkRel {
        &self.rel
    }

    /// Get anchor of this link
    #[inline]
    pub fn anchor(&self) -> Option<&UriReferenceString> {
        self.anchor.as_ref()
    }

    /// Get params of this link
    #[inline]
    pub fn params(&self) -> &FieldParameters {
        &self.params
    }

    /// Converts target from one type to other.
    #[inline]
    pub fn convert_target<TargetUriRef2>(self) -> LinkValue<TargetUriRef2>
    where
        TargetUriRef2: AsRef<UriReferenceStr>,
        TargetUriRef: Into<TargetUriRef2>,
    {
        LinkValue::new(LinkTarget(self.target.0.into()), self.rel, self.anchor)
    }

    /// Convert into parts.
    #[inline]
    pub fn into_parts(self) -> (LinkTarget<TargetUriRef>, FieldParameters) {
        (self.target, self.params)
    }
}

impl LinkValue<UriReferenceString> {
    /// Try to create new [`LinkValue`] from basic params.
    pub fn try_new_basic(
        target_str: impl AsRef<str>,
        rel_type_str: impl AsRef<str>,
    ) -> Result<Self, InvalidEncodedLinkValue> {
        let target = LinkTarget(
            UriReferenceString::try_from(target_str.as_ref())
                .map_err(|_| InvalidEncodedLinkTarget::InvalidUriRef)?,
        );
        let rel_type = rel_type_str.as_ref().parse().map_err(|e| {
            InvalidEncodedLinkValue::InvalidRelParam(InvalidEncodedLinkRel::InvalidRelationType(e))
        })?;

        Ok(Self::new(target, LinkRel::new(rel_type), None))
    }
}

impl<TargetUriRef> LinkValue<TargetUriRef>
where
    TargetUriRef: AsRef<UriReferenceStr>,
{
    /// Create new [`LinkValue`]
    pub fn new(
        target: LinkTarget<TargetUriRef>,
        rel: LinkRel,
        anchor: Option<UriReferenceString>,
    ) -> Self {
        let mut params = FieldParameters::new(Default::default());
        params.push(FieldParameter {
            name: REL_PARAM_NAME.clone(),
            value: rel.str_encode().as_str().try_into().expect("Must be valid"),
        });
        if let Some(anchor_ref) = &anchor {
            params.push(FieldParameter {
                name: ANCHOR_PARAM_NAME.clone(),
                value: anchor_ref.as_str().try_into().expect("Must be valid"),
            });
        }
        Self {
            target,
            params,
            rel,
            anchor,
        }
    }

    /// Push encoded value to string buffer
    #[inline]
    pub(crate) fn push_encoded_str(&self, buffer: &mut String) {
        self.target.push_encoded_str(buffer);
        buffer.push_str("; ");
        self.params.push_encoded_str(buffer);
    }

    /// Encode link value as header string as per rfc
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }

    // TODO add a method to resolve link context
}

impl<TargetUriRef> LinkValue<TargetUriRef>
where
    TargetUriRef: for<'a> TryFrom<&'a str>,
{
    /// Set a link parameter
    pub fn set_param(
        &mut self,
        name: FieldParameterName,
        value: FieldParameterValue,
    ) -> Result<(), InvalidEncodedLinkValue> {
        if REL_PARAM_NAME.eq(&name) {
            self.rel = LinkRel::decode(value.as_ref())?;
        }
        if ANCHOR_PARAM_NAME.eq(&name) {
            self.anchor = Some(UriReferenceString::try_from(value.as_ref())?);
        }
        self.params.push(FieldParameter { name, value });
        Ok(())
    }
}

#[cfg(test)]
mod tests_parse {
    use claims::*;
    use rstest::rstest;

    use super::*;
    use crate::common::field::rules::parameters::tests_parse::assert_matches_param_records;

    pub fn try_link_value(
        link_value_str: &str,
    ) -> Result<LinkValue<UriReferenceString>, InvalidEncodedLinkValue> {
        LinkValue::from_str(link_value_str)
    }

    pub fn assert_valid_link_value(link_value_str: &str) -> LinkValue<UriReferenceString> {
        assert_ok!(try_link_value(link_value_str))
    }

    pub fn assert_link_value_match(
        link_value: &LinkValue<UriReferenceString>,
        expected_target_str: &str,
        expected_params: &[(&str, &str)],
    ) {
        assert_eq!(
            link_value.target(),
            &assert_ok!(LinkTarget::decode(expected_target_str))
        );
        assert_matches_param_records(link_value.params(), expected_params);
    }

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
    fn valid_link_value_will_be_parsed_correctly(
        #[case] link_value_str: &str,
        #[case] expected_target_str: &str,
        #[case] expected_params: &[(&str, &str)],
    ) {
        let link_value = assert_valid_link_value(link_value_str);
        assert_link_value_match(&link_value, expected_target_str, expected_params);
    }

    #[rstest]
    #[case::unenclosed("https://example.org/; rel=previous;")]
    #[case::invalid("<https://example.org/ a>; rel=previous;")]
    #[case::invalid2("<https://example.org/{a}>; rel=previous;")]
    fn link_value_with_invalid_target_will_be_rejected(#[case] link_value_str: &str) {
        assert_matches!(
            assert_err!(try_link_value(link_value_str)),
            InvalidEncodedLinkValue::InvalidLinkTarget(..)
        );
    }

    #[rstest]
    #[case("<https://example.org/>;")]
    #[case("<./>;")]
    fn link_value_with_out_rel_will_be_rejected(#[case] link_value_str: &str) {
        assert_matches!(
            assert_err!(try_link_value(link_value_str)),
            InvalidEncodedLinkValue::NoRelParam
        );
    }

    #[rstest]
    #[case("<https://example.org/>; rel=\"a{b}\"")]
    #[case("<./>; rel=\"b[c]\"")]
    fn link_value_with_invalid_rel_will_be_rejected(#[case] link_value_str: &str) {
        assert_matches!(
            assert_err!(try_link_value(link_value_str)),
            InvalidEncodedLinkValue::InvalidRelParam(..)
        );
    }

    #[rstest]
    #[case("<https://example.org/>; rel")]
    #[case("<./>; rel=type; a=\"abc")]
    fn link_value_with_invalid_params_will_be_rejected(#[case] link_value_str: &str) {
        assert_matches!(
            assert_err!(try_link_value(link_value_str)),
            InvalidEncodedLinkValue::InvalidPFieldValue(..)
        );
    }
}

#[cfg(test)]
mod tests_encode {
    use claims::*;
    use rstest::rstest;

    use super::{tests_parse::*, *};

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
    fn round_trip_works_correctly(
        #[case] link_value_str: &str,
        #[case] expected_target_str: &str,
        #[case] expected_params: &[(&str, &str)],
    ) {
        let link_value = assert_valid_link_value(link_value_str);
        assert_link_value_match(&link_value, expected_target_str, expected_params);

        let link_value_round_tripped =
            assert_ok!(LinkValue::from_str(link_value.str_encode().as_str()));
        assert_link_value_match(
            &link_value_round_tripped,
            expected_target_str,
            expected_params,
        );
    }
}
