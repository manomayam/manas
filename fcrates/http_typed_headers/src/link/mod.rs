//! I define [`Link`] typed header and related structures.
//!
use std::str::FromStr;

use headers::Header;
use iri_string::types::{UriReferenceStr, UriReferenceString};

use crate::common::field::rules::flat_csv::{Comma, FlatCsv};

mod rel;
mod relation_type;
mod target;
mod value;

pub use rel::*;
pub use relation_type::*;
pub use target::*;
pub use value::*;

/// `Link` header is defined in [`rfc8288`](https://datatracker.ietf.org/doc/html/rfc8288)
///
/// The Link header field provides a means for serializing one or more
/// links into HTTP headers.
///
/// The ABNF for the field value is:
///
/// ```txt
///     Link       = #link-value
///     link-value = "<" URI-Reference ">" *( OWS ";" OWS link-param )
///     link-param = token BWS [ "=" BWS ( token / quoted-string ) ]
/// ```
#[derive(Debug, Clone)]
pub struct Link<TargetUriRef = UriReferenceString> {
    /// List of link values.
    pub values: Vec<LinkValue<TargetUriRef>>,
}

impl<TargetUriRef> Default for Link<TargetUriRef> {
    #[inline]
    fn default() -> Self {
        Self { values: vec![] }
    }
}

impl<TargetUriRef> Header for Link<TargetUriRef>
where
    TargetUriRef: for<'a> TryFrom<&'a str> + AsRef<UriReferenceStr>,
{
    fn name() -> &'static headers::HeaderName {
        &http::header::LINK
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let link_values = values
            .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
            .map(LinkValue::<TargetUriRef>::from_str)
            .collect::<Result<Vec<LinkValue<TargetUriRef>>, _>>()
            .map_err(|_| headers::Error::invalid())?;

        Ok(Self {
            values: link_values,
        })
    }

    #[inline]
    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(self.values.iter().map(|v| v.into()));
    }
}

// TODO crud tests
