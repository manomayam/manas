//! I define struct to represent `jti` dpop-proof claim.
//!

use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// A struct representing `jti` dpop-proof claim.
///
/// From spec:
///
/// > Unique identifier for the DPoP proof JWT. The value MUST be
/// > assigned such that there is a negligible probability that the same
/// > value will be assigned to any other DPoP proof used in the
/// > same context during the time window of validity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Jti(String);

impl From<String> for Jti {
    #[inline]
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl From<Jti> for String {
    #[inline]
    fn from(val: Jti) -> Self {
        val.0
    }
}

impl Deref for Jti {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl Jti {
    /// Create a new `jti` from a generated uuid4.
    #[inline]
    pub fn new_from_uuid4() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}
