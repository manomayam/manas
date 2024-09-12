//! I define types to represent dpop-proof jwks.
//!

use std::{borrow::Cow, ops::Deref};

use gdp_rs::{proven::ProvenError, Proven};
use picky::jose::jwk::Jwk;
use serde::{Deserialize, Serialize};

pub use self::predicate::InvalidDPoPProofJwk;
use self::predicate::IsValidDPoPProofJwk;

/// Type of `jwk` header parameter values for DPoPProof JWT.
///
/// From [rfc draft](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-4.2):
///
/// >  representing the public key chosen by the client, in JSON Web Key (JWK) RFC7517 format,
/// as defined in Section 4.1.3 of RFC7515. MUST NOT contain a private key..
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DPoPProofJwk<'inner>(Proven<Cow<'inner, Jwk>, IsValidDPoPProofJwk>);

impl<'inner> TryFrom<Cow<'inner, Jwk>> for DPoPProofJwk<'inner> {
    type Error = ProvenError<Cow<'inner, Jwk>, InvalidDPoPProofJwk>;

    #[inline]
    fn try_from(jwk: Cow<'inner, Jwk>) -> Result<Self, Self::Error> {
        Ok(Self(Proven::try_new(jwk)?))
    }
}

impl TryFrom<Jwk> for DPoPProofJwk<'static> {
    type Error = ProvenError<Jwk, InvalidDPoPProofJwk>;

    #[inline]
    fn try_from(jwk: Jwk) -> Result<Self, Self::Error> {
        Ok(Self(Proven::try_new(Cow::Owned(jwk)).map_err(|e| {
            ProvenError {
                error: e.error,
                subject: (e.subject as Cow<Jwk>).into_owned(),
            }
        })?))
    }
}

impl<'inner> From<DPoPProofJwk<'inner>> for Cow<'inner, Jwk> {
    #[inline]
    fn from(alg: DPoPProofJwk<'inner>) -> Self {
        alg.0.into_subject()
    }
}

impl<'inner> From<DPoPProofJwk<'inner>> for Jwk {
    #[inline]
    fn from(alg: DPoPProofJwk<'inner>) -> Self {
        alg.0.into_subject().into_owned()
    }
}

impl<'inner> Deref for DPoPProofJwk<'inner> {
    type Target = Jwk;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<'inner> DPoPProofJwk<'inner> {
    /// Get a dpop-proof jwk from given jwk without any checks.
    ///
    /// # Safety
    /// Caller must ensure that jwk is dpop-compatible.
    #[inline]
    pub unsafe fn new_unchecked(jwk: Cow<'inner, Jwk>) -> Self {
        Self(Proven::new_unchecked(jwk))
    }
}

mod predicate {
    use std::borrow::{Borrow, Cow};

    use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};
    use picky::jose::jwk::Jwk;

    /// A predicate over [`Jwk`] that claims it to be valid in context of dpop-proof.
    ///
    /// From [rfc-draft](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-4.2):
    ///
    /// > MUST NOT contain a private key..
    #[derive(Debug)]
    pub struct IsValidDPoPProofJwk;

    impl<K: Borrow<Jwk>> Predicate<K> for IsValidDPoPProofJwk {
        fn label() -> Cow<'static, str> {
            "IsValidDPoPProofJwk".into()
        }
    }

    /// A type for invalid dpop proof jwks.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
    pub enum InvalidDPoPProofJwk {
        /// JWK Contains a private key.
        #[error("JWK Contains a private key.")]
        ContainsPrivateKey,
        /// JWK is using an unsupported key type (oct)
        /// Unsupported by underlying picky::jose::jwk crate
        #[error("JWK is using an unsupported key type.")]
        UsingUnsupportedKeyType,
    }

    impl<K: Borrow<Jwk>> SyncEvaluablePredicate<K> for IsValidDPoPProofJwk {
        type EvalError = InvalidDPoPProofJwk;

        fn evaluate_for(jwk: &K) -> Result<(), Self::EvalError> {
            // TODO. Filter out any private keys.
            let borrow_jwk: &Jwk = jwk.borrow();
            match borrow_jwk.key {
                picky::jose::jwk::JwkKeyType::Ed(_) => Ok(()),
                picky::jose::jwk::JwkKeyType::Ec(_) => Ok(()),
                picky::jose::jwk::JwkKeyType::Rsa(_) => Ok(()),
                picky::jose::jwk::JwkKeyType::Oct => Err(InvalidDPoPProofJwk::UsingUnsupportedKeyType),
            }
        }
    }

    impl<K: Borrow<Jwk>> PurePredicate<K> for IsValidDPoPProofJwk {}
}
