//! I define a typed header for `DPoP-Nonce` header.
//!

use headers::{Header, HeaderName, HeaderValue};
use itertools::Itertools;

use crate::proof::payload::nonce::Nonce;

/// A typed header for `DPoP-Nonce` header.
pub struct DPoPNonce(Nonce);

/// Constant for `dpop`-nonce header name.
pub static DPOP_NONCE: HeaderName = HeaderName::from_static("dpop-nonce");

impl Header for DPoPNonce {
    #[inline]
    fn name() -> &'static HeaderName {
        &DPOP_NONCE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let nonce_str = values
            // Ensure that, there is exactly one value.
            .exactly_one()
            .map_err(|_| headers::Error::invalid())?
            .to_str()
            .map_err(|_| headers::Error::invalid())?;

        Ok(Self(
            Nonce::try_from(nonce_str.to_owned()).map_err(|_| headers::Error::invalid())?,
        ))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            HeaderValue::from_str(self.0.as_ref()).expect("Must be valid."),
        ));
    }
}
