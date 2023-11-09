//! I define [`Forwarded`] typed header and related structures.
//!
use std::str::FromStr;

use headers::Header;
use vec1::Vec1;

use crate::common::field::rules::flat_csv::{Comma, FlatCsv};

mod element;
pub use element::*;

/// `Forwarded` header as defined in [`rfc7239`](https://www.rfc-editor.org/rfc/rfc7239#section-4).
///
/// The "Forwarded" HTTP header field is an OPTIONAL header field that,
/// when used, contains a list of parameter-identifier pairs that
/// disclose information that is altered or lost when a proxy is
/// involved in the path of the request.
///
/// ```txt
///        Forwarded   = 1#forwarded-element
/// ```
#[derive(Debug, Clone)]
pub struct Forwarded {
    /// One or more forwarded elements.
    pub elements: Vec1<ForwardedElement>,
}

impl Header for Forwarded {
    #[inline]
    fn name() -> &'static headers::HeaderName {
        &http::header::FORWARDED
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let elements = values
            .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
            .map(ForwardedElement::from_str)
            .collect::<Result<Vec<ForwardedElement>, _>>()
            .map_err(|_| headers::Error::invalid())?;

        Ok(Self {
            elements: elements.try_into().map_err(|_| headers::Error::invalid())?,
        })
    }

    #[inline]
    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(self.elements.iter().map(|e| e.into()));
    }
}
