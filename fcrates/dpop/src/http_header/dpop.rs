//! I define a typed header for `DPoP` header.
//!

use headers::{Header, HeaderName, HeaderValue};
use itertools::Itertools;

use crate::proof::raw::RawDPoPProof;

/// A typed header for `dPoP` header.
pub struct DPoP(pub RawDPoPProof<'static>);

/// Constant for `dpop` header name.
pub static DPOP: HeaderName = HeaderName::from_static("dpop");

impl Header for DPoP {
    #[inline]
    fn name() -> &'static HeaderName {
        &DPOP
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let compact_repr = values
            // Ensure that, there is exactly one value.
            .exactly_one()
            .map_err(|_| headers::Error::invalid())?
            .to_str()
            .map_err(|_| headers::Error::invalid())?;

        Ok(Self(
            RawDPoPProof::decode(compact_repr.to_owned().into())
                .map_err(|_| headers::Error::invalid())?,
        ))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            HeaderValue::from_str(self.0.compact_repr()).expect("Must be valid."),
        ));
    }
}
