//! I define [`X-Forwarded-Proto`] typed header and related structures.
//!
use std::ops::Deref;

use headers::{Header, HeaderName};

use crate::field::rules::token::Token;

/// The X-Forwarded-Proto (XFP) header is a de-facto standard header
/// for identifying the original protocol used by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XForwardedProto(Token);

impl Deref for XForwardedProto {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Static for `x-forwarded-proto` header name.
pub static X_FORWARDED_PROTO: HeaderName = HeaderName::from_static("x-forwarded-proto");

impl Header for XForwardedProto {
    #[inline]
    fn name() -> &'static HeaderName {
        &X_FORWARDED_PROTO
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let protocol = values
            .next()
            .ok_or_else(headers::Error::invalid)?
            .to_str()
            .map_err(|_| headers::Error::invalid())?;

        Ok(Self(
            protocol.parse().map_err(|_| headers::Error::invalid())?,
        ))
    }

    #[inline]
    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            self.0.parse().expect("Must be valid header value"),
        ))
    }
}
