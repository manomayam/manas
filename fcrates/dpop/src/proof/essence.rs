//! I define types to represent essence of dpop-proofs.
//!

use picky::jose::jws::Jws;

use super::{
    header::{DPoPProofHeader, InvalidDPoPProofHeader},
    payload::DPoPProofClaims,
};

/// A struct for representing dpop-proof essence.
#[derive(Debug, Clone)]
pub struct DPoPProofEssence {
    /// Header of the dpop-proof.
    pub header: DPoPProofHeader<'static>,

    /// Claims of the dpop-proof.
    pub claims: DPoPProofClaims,
}

impl From<DPoPProofEssence> for Jws {
    fn from(essence: DPoPProofEssence) -> Self {
        Jws {
            header: essence.header.into(),
            payload: serde_json::to_vec(&essence.claims).expect("Must be json serializable."),
        }
    }
}

impl From<&DPoPProofEssence> for Jws {
    fn from(essence: &DPoPProofEssence) -> Self {
        Jws {
            header: essence.header.clone().into(),
            payload: serde_json::to_vec(&essence.claims).expect("Must be json serializable."),
        }
    }
}

impl TryFrom<Jws> for DPoPProofEssence {
    type Error = InvalidDPoPProofJws;

    fn try_from(jws: Jws) -> Result<Self, Self::Error> {
        // Parse dpop proof header.
        let header = DPoPProofHeader::try_from(jws.header).map_err(|e| e.error)?;

        // Parse dpop proof claims.
        let claims = serde_json::from_slice::<DPoPProofClaims>(&jws.payload)?;

        Ok(DPoPProofEssence { header, claims })
    }
}

/// Type for representing errors of a jws being not a valid dpop-proof.
#[derive(Debug, thiserror::Error)]
pub enum InvalidDPoPProofJws {
    /// Jws header is invalid dpop-proof header.
    #[error("Jws header is invalid dpop-proof header.\n{0}")]
    InvalidHeader(#[from] InvalidDPoPProofHeader),

    /// Jws payload json is invalid.
    #[error("Jws payload json is invalid.\n{0}")]
    InvalidPayloadJson(#[from] serde_json::Error),
}
