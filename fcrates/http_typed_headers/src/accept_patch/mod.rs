//! I define typed header for `Accept-Patch`.
use headers::{Header, HeaderName};
use mime::Mime;
use tracing::error;
use vec1::Vec1;

use super::common::media_type::MediaType;
use crate::common::field::rules::flat_csv::{Comma, FlatCsv};

/// `Accept-Patch` header is defined in [`rfc5789`](https://datatracker.ietf.org/doc/html/rfc5789#section-3.1)
///
/// The presence of the
/// Accept-Patch header in response to any method is an implicit
/// indication that PATCH is allowed on the resource identified by the
/// Request-URI.  The presence of a specific patch document format in
/// this header indicates that that specific format is allowed on the
/// resource identified by the Request-URI.
///
/// ```txt
/// Accept-Patch = "Accept-Patch" ":" 1#media-type
///```
///
/// The Accept-Patch header specifies a comma-separated listing of media-
/// types (with optional parameters) as defined by RFC2616, Section
/// 3.7.
#[derive(Debug, Clone)]
pub struct AcceptPatch {
    /// media-types in accept-patch header.
    pub media_types: Vec1<MediaType>,
}

/// Constant for `accept-patch` header-name.
pub static ACCEPT_PATCH: HeaderName = HeaderName::from_static("accept-patch");

impl Header for AcceptPatch {
    #[inline]
    fn name() -> &'static HeaderName {
        &ACCEPT_PATCH
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        Ok(Self {
            media_types: Vec1::try_from(
                values
                    .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
                    .map(|value_str| value_str.parse())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| headers::Error::invalid())?,
            )
            .map_err(|_| {
                error!("Zero number of accept-patch media types not allowed");
                headers::Error::invalid()
            })?,
        })
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(self.media_types.iter().map(|media_type| {
            media_type
                .as_ref()
                .parse()
                .expect("MediaType is always a valid HeaderValue")
        }));
    }
}

impl AcceptPatch {
    /// Create a new [`AcceptPatch`] header with given media type as the first value.
    pub fn new(first_value: MediaType) -> Self {
        Self {
            media_types: Vec1::new(first_value),
        }
    }

    /// Checks if any of accepted media types matches given media_range, with same essence
    #[inline]
    pub fn has_media_type_with_matching_essence(&self, media_range: &Mime) -> bool {
        self.media_types
            .iter()
            .any(|mt| mt.essence_str() == media_range.essence_str())
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use claims::{assert_err, assert_ok};
    use headers::HeaderValue;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("abc")]
    #[case("image//png;")]
    #[case("text/*")]
    #[case("*/*")]
    #[case("image/png; abc")]
    fn header_with_invalid_media_types_will_be_rejected(#[case] header_value_str: &str) {
        let header_value = assert_ok!(HeaderValue::from_str(header_value_str));
        assert_err!(AcceptPatch::decode(&mut std::iter::once(&header_value)));
    }

    fn assert_correspondence(header_values: &[HeaderValue], accept_patch: &AcceptPatch) {
        assert_eq!(
            accept_patch.media_types.len(),
            header_values.len(),
            "Mismatched length"
        );

        for (i, media_type) in accept_patch.media_types.iter().enumerate() {
            assert_eq!(
                media_type.essence_str(),
                assert_ok!(MediaType::from_str(assert_ok!(header_values[i].to_str())))
                    .essence_str(),
                "Mismatched essence for media_type at `\"{}`\"",
                i
            );
        }
    }

    #[rstest]
    #[case(&["text/html", "text/plain; charset=utf8"])]
    #[case(&["image/png", "image/jpg;"])]
    #[case(&["application/ld+json", "text/turtle"])]
    fn valid_headers_will_be_decoded_correctly(#[case] header_value_strs: &[&str]) {
        let header_values: Vec<HeaderValue> = header_value_strs
            .iter()
            .map(|v| assert_ok!(HeaderValue::from_str(v)))
            .collect();
        let accept_patch = assert_ok!(AcceptPatch::decode(&mut header_values.iter()));
        assert_correspondence(&header_values, &accept_patch);
    }

    #[rstest]
    #[case(&["text/html", "text/plain; charset=utf8"])]
    #[case(&["image/png", "image/jpg;"])]
    #[case(&["application/ld+json", "text/turtle"])]
    fn encode_works_correctly(#[case] media_type_strs: &[&str]) {
        let media_types: Vec<MediaType> = media_type_strs
            .iter()
            .map(|s| assert_ok!(s.parse()))
            .collect();
        let accept_patch = AcceptPatch {
            media_types: assert_ok!(media_types.try_into()),
        };

        let mut header_values = Vec::new();
        accept_patch.encode(&mut header_values);

        assert_correspondence(&header_values, &accept_patch);
    }
}
