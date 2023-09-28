//! I define [LinkTarget`] struct.
//!
use std::ops::Deref;

use iri_string::types::{UriReferenceStr, UriReferenceString};

/// Link target is defined in [`rfc8288`](https://datatracker.ietf.org/doc/html/rfc8288#section-3.1)
///
/// Each link-value conveys one target IRI as a URI-Reference (after
/// conversion to one, if necessary; see RFC3987, Section 3.1) inside
/// angle brackets ("<>").  If the URI-Reference is relative, parsers
/// MUST resolve it as per RFC3986, Section 5.  Note that any base IRI
/// appearing in the message's content is not applied.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinkTarget<TargetUriRef = UriReferenceString>(pub TargetUriRef);

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid link target.
pub enum InvalidEncodedLinkTarget {
    /// Target not enclosed in angle brackets.
    #[error("Link target Uri ref must be enclosed in angle brackets")]
    NotEnclosedInAngleBrackets,

    /// Target value is not a valid uri reference.
    #[error("Link target doesn't represent a valid uri reference")]
    InvalidUriRef,
}

impl<TargetUriRef> Deref for LinkTarget<TargetUriRef> {
    type Target = TargetUriRef;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<TargetUriRef> LinkTarget<TargetUriRef>
where
    TargetUriRef: AsRef<UriReferenceStr>,
{
    /// Push encoded value to string buffer
    #[inline]
    pub(crate) fn push_encoded_str(&self, buffer: &mut String) {
        buffer.push('<');
        buffer.push_str(self.0.as_ref().as_str());
        buffer.push('>');
    }

    /// Encode link target as string
    #[inline]
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }
}

impl<TargetUriRef> LinkTarget<TargetUriRef>
where
    TargetUriRef: for<'a> TryFrom<&'a str>,
{
    /// Decodes header encoded link_target
    pub fn decode(encoded_str: &str) -> Result<Self, InvalidEncodedLinkTarget> {
        // Check if reference is enclosed in angle brackets
        if !encoded_str.starts_with('<') || !encoded_str.ends_with('>') {
            return Err(InvalidEncodedLinkTarget::NotEnclosedInAngleBrackets);
        }
        let target_str = &encoded_str[1..encoded_str.len() - 1];

        Ok(Self(
            target_str
                .try_into()
                .map_err(|_| InvalidEncodedLinkTarget::InvalidUriRef)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use claims::*;
    use iri_string::types::UriReferenceString;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("<>", "")]
    #[case("</>", "/")]
    #[case(
        "<http://example.com/TheBook/chapter2>",
        "http://example.com/TheBook/chapter2"
    )]
    #[case("<http://example.net/foo>", "http://example.net/foo")]
    #[case("</terms>", "/terms")]
    #[case("</TheBook/chapter2>", "/TheBook/chapter2")]
    #[case("<http://example.org/>", "http://example.org/")]
    #[case("<abc>", "abc")]
    fn valid_link_target_will_be_decoded_correctly(
        #[case] encoded_link_target_str: &str,
        #[case] expected_uri_ref_str: &str,
    ) {
        let link_target = assert_ok!(LinkTarget::<UriReferenceString>::decode(
            encoded_link_target_str
        ));
        assert_eq!(
            link_target.as_str(),
            expected_uri_ref_str,
            "uri ref is parsed incorrectly"
        );
    }

    #[rstest]
    #[case("a/")]
    #[case("http://example.org/a")]
    fn unenclosed_link_target_will_be_rejected(#[case] encoded_link_target_str: &str) {
        assert_err_eq!(
            LinkTarget::<UriReferenceString>::decode(encoded_link_target_str),
            InvalidEncodedLinkTarget::NotEnclosedInAngleBrackets
        );
    }

    #[rstest]
    #[case("<a b>")]
    fn invalid_ref_link_target_will_be_rejected(#[case] encoded_link_target_str: &str) {
        assert_matches!(
            assert_err!(LinkTarget::<UriReferenceString>::decode(
                encoded_link_target_str
            )),
            InvalidEncodedLinkTarget::InvalidUriRef
        );
    }
}
