//! I define [`AcceptPut`] typed header and related structures.
//!
use headers::{Header, HeaderName};
use mime::Mime;

use crate::common::field::rules::flat_csv::{Comma, FlatCsv};

/// `Accept-Put` header is defined in [`solid-protocol`](https://solidproject.org/ED/protocol#accept-put)
///
/// 7.1.1 The syntax for Accept-Put, using the ABNF syntax defined in Section 1.2 of RFC7231, is:
///```txt
/// Accept-Put = "Accept-Put" ":" # media-range
///
/// The Accept-Put header specifies a comma-separated list of media ranges (with optional parameters) as defined by RFC7231, Section 5.3.2. The Accept-Put header, in effect, uses the same syntax as the HTTP Accept header minus the optional accept-params BNF production, since the latter does not apply to Accept-Put.
///```
#[derive(Debug, Clone)]
pub struct AcceptPut {
    /// List of media ranges in accept-put header.
    pub media_ranges: Vec<Mime>,
}

/// Constant for `accept-post` header name.
pub static ACCEPT_PUT: HeaderName = HeaderName::from_static("accept-put");

impl Header for AcceptPut {
    #[inline]
    fn name() -> &'static HeaderName {
        &ACCEPT_PUT
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        Ok(AcceptPut {
            media_ranges: values
                .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
                .map(|value_str| value_str.parse())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| headers::Error::invalid())?,
        })
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(self.media_ranges.iter().map(|media_range| {
            media_range
                .as_ref()
                .parse()
                .expect("Mime is always a valid HeaderValue")
        }));
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
    #[case("image/png; abc")]
    fn header_with_invalid_media_ranges_will_be_rejected(#[case] header_value_str: &str) {
        let header_value = assert_ok!(HeaderValue::from_str(header_value_str));
        assert_err!(AcceptPut::decode(&mut std::iter::once(&header_value)));
    }

    fn assert_correspondence(header_values: &[HeaderValue], accept_put: &AcceptPut) {
        assert_eq!(
            accept_put.media_ranges.len(),
            header_values.len(),
            "Mismatched length"
        );

        for (i, media_type) in accept_put.media_ranges.iter().enumerate() {
            assert_eq!(
                media_type.essence_str(),
                assert_ok!(Mime::from_str(assert_ok!(header_values[i].to_str()))).essence_str(),
                "Mismatched essence for media_range at `\"{}`\"",
                i
            );
        }
    }

    #[rstest]
    #[case(&["text/html", "text/plain; charset=utf8"])]
    #[case(&["image/png", "image/jpg;"])]
    #[case(&["application/ld+json", "text/turtle"])]
    #[case(&["image/*", "text/*"])]
    #[case(&["*/*", "text/turtle"])]
    fn valid_headers_will_be_decoded_correctly(#[case] header_value_strs: &[&str]) {
        let header_values: Vec<HeaderValue> = header_value_strs
            .iter()
            .map(|v| assert_ok!(HeaderValue::from_str(v)))
            .collect();
        let accept_put = assert_ok!(AcceptPut::decode(&mut header_values.iter()));
        assert_correspondence(&header_values, &accept_put);
    }

    #[rstest]
    #[case(&["text/html", "text/plain; charset=utf8"])]
    #[case(&["image/png", "image/jpg;"])]
    #[case(&["application/ld+json", "text/turtle"])]
    fn encode_works_correctly(#[case] media_range_strs: &[&str]) {
        let accept_put = AcceptPut {
            media_ranges: media_range_strs
                .iter()
                .map(|s| assert_ok!(s.parse()))
                .collect(),
        };

        let mut header_values = Vec::new();
        accept_put.encode(&mut header_values);

        assert_correspondence(&header_values, &accept_put);
    }
}
