//! I define [`X-Forwarded-Host] typed header and related structures.
//!
use std::ops::Deref;

use headers::{Header, HeaderName, Host};

/// The X-Forwarded-Host (XFH) header is a de-facto standard header
/// for identifying the original host requested by the client
/// in the Host HTTP request header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XForwardedHost(pub Host);

impl Deref for XForwardedHost {
    type Target = Host;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<XForwardedHost> for Host {
    #[inline]
    fn from(val: XForwardedHost) -> Self {
        val.0
    }
}

/// Static for `x-forwarded-host` header name.
pub static X_FORWARDED_HOST: HeaderName = HeaderName::from_static("x-forwarded-host");

impl Header for XForwardedHost {
    #[inline]
    fn name() -> &'static HeaderName {
        &X_FORWARDED_HOST
    }

    #[inline]
    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        Ok(Self(Host::decode(values)?))
    }

    #[inline]
    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        self.0.encode(values)
    }
}
