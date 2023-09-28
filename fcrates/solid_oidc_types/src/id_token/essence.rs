//! I define types to represent essence of id-tokens.
//!

use picky::jose::jws::Jws;

use super::{header::IdTokenHeader, payload::IdTokenClaims};

/// A struct for representing id-token essence.
#[derive(Debug, Clone)]
pub struct IdTokenEssence {
    /// Header of the id-token.
    pub header: IdTokenHeader,

    /// Claims of the id-token.
    pub claims: IdTokenClaims,
}

impl From<IdTokenEssence> for Jws {
    fn from(essence: IdTokenEssence) -> Self {
        Jws {
            header: essence.header,
            payload: serde_json::to_vec(&essence.claims).expect("Must be json serializable."),
        }
    }
}

impl From<&IdTokenEssence> for Jws {
    fn from(essence: &IdTokenEssence) -> Self {
        Jws {
            header: essence.header.clone(),
            payload: serde_json::to_vec(&essence.claims).expect("Must be json serializable."),
        }
    }
}

impl TryFrom<Jws> for IdTokenEssence {
    type Error = InvalidIdTokenJws;

    fn try_from(jws: Jws) -> Result<Self, Self::Error> {
        // Parse dpop proof header.
        let header = jws.header;

        // Parse dpop proof claims.
        let claims = serde_json::from_slice::<IdTokenClaims>(&jws.payload)?;

        Ok(IdTokenEssence { header, claims })
    }
}

/// Type for representing errors of a jws being not a valid id-token.
#[derive(Debug, thiserror::Error)]
pub enum InvalidIdTokenJws {
    /// Jws payload json is invalid.
    #[error("Jws payload json is invalid.\n{0}")]
    InvalidPayloadJson(#[from] serde_json::Error),
}
