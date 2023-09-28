//! I define types to represent dpop-proof algorithms.
//!

use gdp_rs::{proven::ProvenError, Proven};
use picky::jose::jws::JwsAlg;
use serde::{Deserialize, Serialize};

pub use self::predicate::InvalidDPoPProofAlg;
use self::predicate::IsValidDPoPProofAlg;

/// Type of `alg` header parameter values for DPoPProof JWT.
///
/// From [rfc draft](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-4.2):
///
/// >  alg: an identifier for a JWS asymmetric digital signature algorithm from [IANA.JOSE.ALGS].
/// > MUST NOT be none or an identifier for a symmetric algorithm (MAC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DPoPProofAlg(Proven<JwsAlg, IsValidDPoPProofAlg>);

impl TryFrom<JwsAlg> for DPoPProofAlg {
    type Error = ProvenError<JwsAlg, InvalidDPoPProofAlg>;

    #[inline]
    fn try_from(alg: JwsAlg) -> Result<Self, Self::Error> {
        Ok(Self(Proven::try_new(alg)?))
    }
}

impl From<DPoPProofAlg> for JwsAlg {
    #[inline]
    fn from(alg: DPoPProofAlg) -> Self {
        alg.0.into_subject()
    }
}

impl DPoPProofAlg {
    /// Get a dpop-proof alg from jws alg without any checks.
    ///
    /// # Safety
    /// Caller must ensure that alg is dpop-compatible.
    #[inline]
    pub unsafe fn new_unchecked(alg: JwsAlg) -> Self {
        Self(Proven::new_unchecked(alg))
    }
}

mod predicate {
    use std::borrow::{Borrow, Cow};

    use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};
    use picky::jose::jws::JwsAlg;

    /// A predicate over [`JwsAlg`] that claims it to be valid in context of dpop-proof.
    ///
    /// From [rfc-draft](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-4.2):
    ///
    /// >  alg: an identifier for a JWS asymmetric digital signature algorithm from [IANA.JOSE.ALGS].
    /// > MUST NOT be none or an identifier for a symmetric algorithm (MAC).
    #[derive(Debug)]
    pub struct IsValidDPoPProofAlg;

    impl<Alg: Borrow<JwsAlg>> Predicate<Alg> for IsValidDPoPProofAlg {
        fn label() -> Cow<'static, str> {
            "IsValidDPoPProofAlg".into()
        }
    }

    /// A type for invalid dpop proof algorithm errors.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum InvalidDPoPProofAlg {
        /// Alg is None.
        #[error("Alg is None.")]
        IsNone,

        /// Alg is symmetric.
        #[error("Alg is symmetric.")]
        IsSymmetric,
    }

    impl<Alg: Borrow<JwsAlg>> SyncEvaluablePredicate<Alg> for IsValidDPoPProofAlg {
        type EvalError = InvalidDPoPProofAlg;

        fn evaluate_for(alg: &Alg) -> Result<(), Self::EvalError> {
            let alg = alg.borrow();
            match alg {
                JwsAlg::HS256 => Err(InvalidDPoPProofAlg::IsSymmetric),
                JwsAlg::HS384 => Err(InvalidDPoPProofAlg::IsSymmetric),
                JwsAlg::HS512 => Err(InvalidDPoPProofAlg::IsSymmetric),
                JwsAlg::RS256 => Ok(()),
                JwsAlg::RS384 => Ok(()),
                JwsAlg::RS512 => Ok(()),
                JwsAlg::ES256 => Ok(()),
                JwsAlg::ES384 => Ok(()),
                JwsAlg::ES512 => Ok(()),
                JwsAlg::PS256 => Ok(()),
                JwsAlg::PS384 => Ok(()),
                JwsAlg::PS512 => Ok(()),
                JwsAlg::EdDSA => Ok(()),
                #[allow(deprecated)]
                JwsAlg::ED25519 => Ok(()),
            }
        }
    }

    impl<Alg: Borrow<JwsAlg>> PurePredicate<Alg> for IsValidDPoPProofAlg {}
}
