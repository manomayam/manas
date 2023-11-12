//! I define [`Location`] typed header.
//!

use std::str::FromStr;

use headers::{Header, HeaderName, HeaderValue};
use iri_string::types::UriReferenceString;
use itertools::Itertools;
use tracing::error;

/// `Location` header is defined in [`rfc9110`](https://www.rfc-editor.org/rfc/rfc9110.html#section-10.2.2)
///
/// The syntax of the Location header:
///
/// ```txt
///     Location = URI-reference
///```
///  The field value consists of a single URI-reference.
/// When it has the form of a relative reference,
/// the final value is computed by resolving it against the
///  target URI.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location(pub UriReferenceString);

/// Static for `location` header-name.
pub static LOCATION: HeaderName = HeaderName::from_static("location");

impl Header for Location {
    #[inline]
    fn name() -> &'static HeaderName {
        &LOCATION
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let ref_str = values
            .exactly_one()
            .map_err(|_| {
                error!("Invalid number of `Location` header fields.");
                headers::Error::invalid()
            })
            .and_then(|val| {
                val.to_str().map_err(|_| {
                    error!("Invalid `Location` header field.");
                    headers::Error::invalid()
                })
            })?;

        Ok(Self(UriReferenceString::from_str(ref_str).map_err(
            |_| {
                error!("Invalid `Location` header field.");
                headers::Error::invalid()
            },
        )?))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            self.0
                .as_str()
                .try_into()
                .expect("Must be a valid header value."),
        ));
    }
}
