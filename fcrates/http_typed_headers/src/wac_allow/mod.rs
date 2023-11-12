//! I define `wac-allow` typed header and related types

mod access_mode;
mod access_param;
mod permission_group;

pub use access_mode::*;
pub use access_param::*;
use headers::Header;
use http::{HeaderName, HeaderValue};
pub use permission_group::*;

use crate::common::field::rules::flat_csv::{Comma, FlatCsv};

/// Typed header for `Wac-Allow`, as defined in
/// [WAC specification](https://solid.github.io/web-access-control-spec/#wac-allow)
#[derive(Debug, Clone)]
pub struct WacAllow {
    /// List of access params.
    pub access_params: Vec<AccessParam>,
}

/// `Wac=Allow` header name.
pub static WAC_ALLOW: HeaderName = HeaderName::from_static("wac-allow");

impl Header for WacAllow {
    #[inline]
    fn name() -> &'static HeaderName {
        &WAC_ALLOW
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let access_params = values
            .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
            .map(AccessParam::decode)
            .collect::<Result<_, _>>()
            .map_err(|_| headers::Error::invalid())?;
        Ok(Self { access_params })
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let mut buf = String::new();
        self.access_params.iter().for_each(|p| {
            p.push_encoded_str(&mut buf);
            buf.push(',');
        });
        buf.pop(); // trailing comma

        values.extend(std::iter::once(unsafe {
            HeaderValue::from_maybe_shared_unchecked(buf.into_bytes())
        }))
    }
}

// TODO tests
