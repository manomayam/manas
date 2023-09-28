//! I define types for representing DPoP credentials
//! in `Authorization` header.
//!

use bytes::Bytes;
use headers::{authorization::Credentials, HeaderValue};
use token68::Token68;

mod token68;

/// An implementation of [`Credentials`] for DPoP authentication scheme.
///
/// From [rfc](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-7.1)
///
/// ```txt
///   token68    = 1*( ALPHA / DIGIT /
///                    "-" / "." / "_" / "~" / "+" / "/" ) *"="
///
///  credentials = "DPoP" 1*SP token68
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DPoPAuthorizationCredentials {
    /// DPoP bound access token.
    pub access_token: Token68,
}

impl DPoPAuthorizationCredentials {
    /// Create a new [`DPoPAuthorizationCredentials`] with given access token.
    pub fn new(access_token: Token68) -> Self {
        Self { access_token }
    }
}

impl Credentials for DPoPAuthorizationCredentials {
    const SCHEME: &'static str = "DPoP";

    fn decode(value: &HeaderValue) -> Option<Self> {
        debug_assert!(
            value.as_bytes().starts_with(b"DPoP "),
            "HeaderValue to decode should start with \"{} ..\", received = {:?}",
            Self::SCHEME,
            value,
        );

        let access_token = value
            .to_str()
            .ok()
            .and_then(|v| v[Self::SCHEME.len() + 1..].parse().ok())?;

        Some(Self::new(access_token))
    }

    fn encode(&self) -> HeaderValue {
        // TODO use the below pattern for remaining typed headers in manas_http.
        let encoded = Bytes::from(format!("{} {}", Self::SCHEME, self.access_token.as_str()));
        HeaderValue::from_maybe_shared(encoded)
            .expect("base64 encoding is always a valid HeaderValue.")
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_none, assert_some};
    use headers::{authorization::Credentials, HeaderValue};
    use rstest::*;

    use super::DPoPAuthorizationCredentials;

    #[rstest]
    #[case("DPoP fpKL54jvWmEGVoRdCNjG", "fpKL54jvWmEGVoRdCNjG")]
    #[case(
        "DPoP Kz~8mXK1EalYznwH-LC-1fBAo.4Ljp~zsPE_NeO.gxU",
        "Kz~8mXK1EalYznwH-LC-1fBAo.4Ljp~zsPE_NeO.gxU"
    )]
    #[case(
        "DPoP Kz~8mXK1EalYznwH-LC-1fBAo.4Ljp~zsPE_NeO.gxU==",
        "Kz~8mXK1EalYznwH-LC-1fBAo.4Ljp~zsPE_NeO.gxU=="
    )]
    fn valid_creds_roundtrip_works_correctly(
        #[case] header_value_str: &str,
        #[case] access_token_str: &str,
    ) {
        let header_value = HeaderValue::from_str(header_value_str).expect("Invalid header value.");
        let decoded = assert_some!(DPoPAuthorizationCredentials::decode(&header_value));
        assert_eq!(decoded.access_token.as_str(), access_token_str);

        let round_tripped = decoded.encode();
        assert_eq!(
            round_tripped.to_str().expect("Must be string"),
            header_value_str
        );
    }

    #[rstest]
    #[case("")]
    #[case("DPoP")]
    #[case("Bearer abbcg7sd3jh3~d")]
    #[should_panic]
    fn creds_with_invalid_scheme_cause_panic(#[case] header_value_str: &str) {
        let header_value = HeaderValue::from_str(header_value_str).expect("Invalid header value.");
        assert_none!(DPoPAuthorizationCredentials::decode(&header_value));
    }

    #[rstest]
    #[case("DPoP ")]
    #[case("DPoP a bv")]
    #[case("DPoP a=bcd")]
    fn invalid_creds_will_be_rejected(#[case] header_value_str: &str) {
        let header_value = HeaderValue::from_str(header_value_str).expect("Invalid header value.");
        assert_none!(DPoPAuthorizationCredentials::decode(&header_value));
    }
}
