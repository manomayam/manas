//! I define typed `Authorization` header with dpop credentials.
//!

use headers::Authorization;

use self::credentials::DPoPAuthorizationCredentials;

pub mod credentials;

/// Alias for type of [`Authorization`] header with dpop credentials.
pub type DPoPAuthorization = Authorization<DPoPAuthorizationCredentials>;
