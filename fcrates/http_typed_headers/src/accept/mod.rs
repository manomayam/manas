//! I define [`Accept`] typed header and related structs.
//!

use std::ops::Deref;

use headers::Header;
use tracing::error;

use crate::common::field::rules::flat_csv::{Comma, FlatCsv};

mod accept_value;
mod precedence;

pub use accept_value::*;
pub use precedence::*;

/// `Accept` header, defined in
/// [RFC7231](https://datatracker.ietf.org/doc/html/rfc7231#section-5.3.2).
///
/// The "Accept" header field can be used by user agents to specify
/// response media types that are acceptable.  Accept header fields can
/// be used to indicate that the request is specifically limited to a
/// small set of desired types, as in the case of a request for an
/// in-line image.
///
/// ```txt
///   Accept = #( media-range [ accept-params ] )
///
///   media-range    = ( "*/*"
///                    / ( type "/" "*" )
///                    / ( type "/" subtype )
///                    ) *( OWS ";" OWS parameter )
///   accept-params  = weight *( accept-ext )
///   accept-ext = OWS ";" OWS token [ "=" ( token / quoted-string ) ]
///```
#[derive(Clone, Debug)]
pub struct Accept {
    /// List of accept-values.s
    pub accept_values: Vec<AcceptValue>,
}

impl Header for Accept {
    fn name() -> &'static headers::HeaderName {
        &http::header::ACCEPT
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        Ok(Self {
            accept_values: values
                .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
                .map(|value_str| value_str.parse())
                .collect::<Result<_, _>>()
                .map_err(|e| {
                    error!("Error in parsing Accept header. Error:\n {}", e);
                    headers::Error::invalid()
                })?,
        })
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(self.accept_values.iter().map(|accept_value| {
            accept_value
                .deref()
                .as_ref()
                .parse()
                .expect("accept value is always a valid HeaderValue")
        }));
    }
}

impl Accept {
    /// Sorts accept values from highest precedence to lowest
    #[inline]
    pub fn sort_accept_values_by_precedence(&mut self) {
        // Sorts stably in descending order of precedence.
        self.accept_values
            .sort_by(|v1, v2| v2.precedence().cmp(v1.precedence()));
    }
}

#[cfg(test)]
mod tests_decode {
    use claims::assert_ok;
    use headers::HeaderValue;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        &["audio/*; q=0.2", "audio/basic"],
        &["audio/basic", "audio/*; q=0.2"]
    )]
    #[case(
        &["text/*", "text/plain", "text/plain;format=flowed", "*/*"],
        &["text/plain;format=flowed", "text/plain", "text/*", "*/*"]
    )]
    #[case(
        &["text/*;q=0.3", "text/html;q=0.7", "text/html;level=1", "text/html;level=2;q=0.4", "*/*;q=0.5"],
        &["text/html;level=1", "text/html;q=0.7", "*/*;q=0.5", "text/html;level=2;q=0.4", "text/*;q=0.3"]
    )]
    fn valid_header_values_will_be_encoded_correctly(
        #[case] header_value_strs: &[&str],
        #[case] expected_precedence_order: &[&str],
    ) {
        let header_values: Vec<HeaderValue> = header_value_strs
            .iter()
            .map(|v| assert_ok!(v.parse(), "Invalid header value"))
            .collect();

        let mut accept = assert_ok!(
            Accept::decode(&mut header_values.iter()),
            "Invalid Accept header value"
        );

        accept.sort_accept_values_by_precedence();

        let sorted_accept_values = accept.accept_values;
        assert_eq!(sorted_accept_values.len(), expected_precedence_order.len());

        for (i, accept_value) in sorted_accept_values.iter().enumerate() {
            let expected_accept_value: AcceptValue =
                expected_precedence_order[i].parse().unwrap_or_else(|_| {
                    panic!(
                        "Invalid expected media range str '{}'",
                        expected_precedence_order[i]
                    )
                });
            assert_eq!(accept_value.deref(), expected_accept_value.deref());
        }
    }
}
