//! I define types for representing http uri secure transport policy.

use std::fmt::Debug;

use crate::{HttpUri, HTTPS_SCHEME};

/// A trait for representing secure transport policies
/// regarding http uris.
pub trait SecureTransportPolicy: Debug + Send + Sync + 'static {
    /// Check if given uri is secure, as per the policy.
    fn is_secure(uri: &HttpUri) -> bool;
}

/// An implementation of [`SecureTransportPolicy`], that only allows https uris.
#[derive(Debug, Clone)]
pub struct StrictSTP;

impl SecureTransportPolicy for StrictSTP {
    #[inline]
    fn is_secure(uri: &HttpUri) -> bool {
        HTTPS_SCHEME.eq_ignore_ascii_case(uri.scheme_str())
    }
}

/// An implementation of [`SecureTransportPolicy`], that only allows any uris.
pub type VoidSTP = ();

impl SecureTransportPolicy for VoidSTP {
    #[inline]
    fn is_secure(_uri: &HttpUri) -> bool {
        true
    }
}

/// An implementation of [`SecureTransportPolicy`], that only allows
/// https uris and localhost uris with any(http/https) scheme..
#[derive(Debug, Clone)]
pub struct LocalhostExemptingSTP;

impl SecureTransportPolicy for LocalhostExemptingSTP {
    #[inline]
    fn is_secure(uri: &HttpUri) -> bool {
        uri.is_https() || uri.is_localhost()
    }
}
