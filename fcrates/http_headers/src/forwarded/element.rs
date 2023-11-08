//! I define [`ForwardedElement] structure
//! corresponding to `forwarded-element` production.
//!
use std::{ops::Deref, str::FromStr};

use headers::{HeaderValue, Host};
use http::uri::Authority;
use once_cell::sync::Lazy;

use crate::common::field::rules::{
    flat_csv::{split_field_params, SemiColon},
    parameter_name::FieldParameterName,
    parameter_value::FieldParameterValue,
    parameters::{FieldParameters, InvalidEncodedFieldParameters},
};

/// An element in `Forwarded` header, as defined in [`rfc7239`](https://www.rfc-editor.org/rfc/rfc7239#section-4).
///
/// ```txt
///        forwarded-element =
///        [ forwarded-pair ] *( ";" [ forwarded-pair ] )
///
///    forwarded-pair = token "=" value
///    value          = token / quoted-string
///
///    token = <Defined in [RFC7230], Section 3.2.6>
///    quoted-string = <Defined in [RFC7230], Section 3.2.6>
///
#[derive(Debug, Clone)]
pub struct ForwardedElement {
    /// List of field parameters.
    pub params: FieldParameters,
}

impl FromStr for ForwardedElement {
    type Err = InvalidEncodedFieldParameters;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // Get forwarded pairs.
        let forwarded_pairs = split_field_params::<SemiColon>(value);

        let params = FieldParameters::decode(forwarded_pairs, false)?;

        Ok(Self { params })
    }
}

impl From<&ForwardedElement> for HeaderValue {
    #[inline]
    fn from(element: &ForwardedElement) -> Self {
        HeaderValue::from_str(element.str_encode().as_str()).expect("Must be valid header value")
    }
}

impl ForwardedElement {
    /// Push encoded value to string buffer
    #[inline]
    pub fn push_encoded_str(&self, buffer: &mut String) {
        self.params.push_encoded_str(buffer);
    }

    /// Encode link value as header string as per rfc
    pub fn str_encode(&self) -> String {
        let mut encoded = String::new();
        self.push_encoded_str(&mut encoded);
        encoded
    }

    /// Get forwarded host.
    #[inline]
    pub fn host(&self) -> Option<&FieldParameterValue> {
        self.params.get_value(FWD_PARAM_HOST.deref())
    }

    /// Get forwarded host decoded.
    pub fn host_decoded(&self) -> Option<Host> {
        self.host()
            .and_then(|host_val| Authority::from_str(host_val).ok())
            .map(|authority| authority.into())
    }

    /// Get forwarded proto.
    #[inline]
    pub fn proto(&self) -> Option<&FieldParameterValue> {
        self.params.get_value(FWD_PARAM_PROTO.deref())
    }
}

/// The "by" parameter is used to disclose the interface where the
/// request came in to the proxy server.
pub static FWD_PARAM_BY: Lazy<FieldParameterName> =
    Lazy::new(|| "by".parse().expect("Must be valid param key."));

/// The "for" parameter is used to disclose information about the client
/// that initiated the request and subsequent proxies in a chain of proxies.
pub static FWD_PARAM_FOR: Lazy<FieldParameterName> =
    Lazy::new(|| "for".parse().expect("Must be valid param key."));

/// The "host" parameter is used to forward the original value of the "Host" header field.
pub static FWD_PARAM_HOST: Lazy<FieldParameterName> =
    Lazy::new(|| "host".parse().expect("Must be valid param key."));

/// The "proto" parameter has the value of the used protocol type.  The
pub static FWD_PARAM_PROTO: Lazy<FieldParameterName> =
    Lazy::new(|| "proto".parse().expect("Must be valid param key."));
