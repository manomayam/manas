//! I define type to represent dpop-proof header.
//!

use std::{borrow::Cow, ops::Deref};

use gdp_rs::{proven::ProvenError, Proven};
use picky::jose::jws::JwsHeader;
pub use predicate::InvalidDPoPProofHeader;
use predicate::IsValidDPoPProofHeader;
use serde::{Deserialize, Serialize};

use self::{alg::DPoPProofAlg, jwk::DPoPProofJwk};

pub mod alg;
pub mod jwk;

///  > typ: with value dpop+jwt, which explicitly types the DPoP proof JWT.
pub const DPOP_PROOF_TYPE: &str = "dpop+jwt";

/// A struct for representing dpop-proof jwt's header.
///
/// A dpop-proof's jose header MUST contain `typ`, `alg`, `jwk` fields with appropriate values.
///
/// See [4.2. DPoP Proof JWT Syntax](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-dpop#section-4.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DPoPProofHeader<'inner>(Proven<Cow<'inner, JwsHeader>, IsValidDPoPProofHeader>);

impl<'inner> TryFrom<Cow<'inner, JwsHeader>> for DPoPProofHeader<'inner> {
    type Error = ProvenError<Cow<'inner, JwsHeader>, InvalidDPoPProofHeader>;

    #[inline]
    fn try_from(jwk: Cow<'inner, JwsHeader>) -> Result<Self, Self::Error> {
        Ok(Self(Proven::try_new(jwk)?))
    }
}

impl<'inner> TryFrom<&'inner JwsHeader> for DPoPProofHeader<'inner> {
    type Error = InvalidDPoPProofHeader;

    fn try_from(value: &'inner JwsHeader) -> Result<Self, Self::Error> {
        Self::try_from(Cow::Borrowed(value)).map_err(|e| e.error)
    }
}

impl TryFrom<JwsHeader> for DPoPProofHeader<'static> {
    type Error = ProvenError<JwsHeader, InvalidDPoPProofHeader>;

    #[inline]
    fn try_from(header: JwsHeader) -> Result<Self, Self::Error> {
        Ok(Self(Proven::try_new(Cow::Owned(header)).map_err(|e| {
            ProvenError {
                error: e.error,
                subject: (e.subject as Cow<JwsHeader>).into_owned(),
            }
        })?))
    }
}

impl<'inner> From<DPoPProofHeader<'inner>> for Cow<'inner, JwsHeader> {
    #[inline]
    fn from(alg: DPoPProofHeader<'inner>) -> Self {
        alg.0.into_subject()
    }
}

impl<'inner> From<DPoPProofHeader<'inner>> for JwsHeader {
    #[inline]
    fn from(alg: DPoPProofHeader<'inner>) -> Self {
        alg.0.into_subject().into_owned()
    }
}

impl<'inner> Deref for DPoPProofHeader<'inner> {
    type Target = JwsHeader;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<'inner> DPoPProofHeader<'inner> {
    /// Get a dpop-proof header from given jose header without any checks.
    ///
    /// # Safety
    /// Caller must ensure that header is dpop-compatible.
    #[inline]
    pub unsafe fn new_unchecked(header: Cow<'inner, JwsHeader>) -> Self {
        Self(Proven::new_unchecked(header))
    }

    /// Get an identical borrowed [`DPoPProofHeader`].
    #[inline]
    pub fn to_borrowed(&self) -> DPoPProofHeader<'_> {
        unsafe { DPoPProofHeader::new_unchecked(Cow::Borrowed(self.0.as_ref())) }
    }

    /// Convert into owned [`DPoPProofHeader`].
    #[inline]
    pub fn into_owned(self) -> DPoPProofHeader<'static> {
        unsafe { DPoPProofHeader::new_unchecked(Cow::Owned(self.0.into_subject().into_owned())) }
    }

    /// Get `typ` param value from header.
    #[inline]
    pub fn typ(&self) -> &str {
        DPOP_PROOF_TYPE
    }

    /// Get `alg` param value from header.
    #[inline]
    pub fn alg(&self) -> DPoPProofAlg {
        // Safety: Checked at the time of instantiation.
        unsafe { DPoPProofAlg::new_unchecked(self.0.alg) }
    }

    /// Get `jwk` param value from header.
    #[inline]
    pub fn jwk(&self) -> DPoPProofJwk<'_> {
        // Safety: Checked at the time of instantiation.
        unsafe {
            DPoPProofJwk::new_unchecked(Cow::Borrowed(
                self.0
                    .jwk
                    .as_ref()
                    .expect("Must be some, as checked at instantiation"),
            ))
        }
    }
}

mod predicate {
    use std::borrow::{Borrow, Cow};

    use gdp_rs::{
        predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
        proven::ProvenError,
    };
    use picky::jose::jws::JwsHeader;

    use super::{
        alg::{DPoPProofAlg, InvalidDPoPProofAlg},
        jwk::{DPoPProofJwk, InvalidDPoPProofJwk},
        DPOP_PROOF_TYPE,
    };

    /// A predicate over [`JwsHeader`] that claims it to be valid in context of dpop-proof.
    #[derive(Debug)]
    pub struct IsValidDPoPProofHeader;

    impl<H: Borrow<JwsHeader>> Predicate<H> for IsValidDPoPProofHeader {
        fn label() -> Cow<'static, str> {
            "IsValidDPoPProofHeader".into()
        }
    }

    /// An enum representing errors in parsing dpop-proof header from a jose header.
    #[derive(Debug, Clone, thiserror::Error)]
    pub enum InvalidDPoPProofHeader {
        /// Required param is absent in given jose header.
        #[error("Required param \"{0}\" is absent in given jose header.")]
        RequiredParamIsAbsent(String),

        /// Invalid dpop-proof typ in jose header.
        #[error("Invalid dpop-proof typ in jose header.")]
        InvalidTyp,

        /// Invalid dpop-proof alg in jose header.
        #[error("Invalid dpop-proof alg in jose header.")]
        InvalidAlg(#[from] InvalidDPoPProofAlg),

        /// Invalid dpop-proof jwl in jose header.
        #[error("Invalid dpop-proof jwk in jose header.")]
        InvalidJwk(#[from] InvalidDPoPProofJwk),
    }

    impl<H: Borrow<JwsHeader>> SyncEvaluablePredicate<H> for IsValidDPoPProofHeader {
        type EvalError = InvalidDPoPProofHeader;

        fn evaluate_for(header_cow: &H) -> Result<(), Self::EvalError> {
            let header: &JwsHeader = header_cow.borrow();

            // Check `typ` param.
            if let Some(typ) = header.typ.as_deref() {
                if typ != DPOP_PROOF_TYPE {
                    return Err(InvalidDPoPProofHeader::InvalidTyp);
                }
            } else {
                return Err(InvalidDPoPProofHeader::RequiredParamIsAbsent(
                    "typ".to_owned(),
                ));
            }

            // Check `alg` param.
            if let Err(e) = DPoPProofAlg::try_from(header.alg) {
                return Err(InvalidDPoPProofHeader::InvalidAlg(e.error));
            }

            // Check `typ` param.
            if let Some(jwk) = header.jwk.as_ref() {
                if let Err(ProvenError { error, .. }) = DPoPProofJwk::try_from(Cow::Borrowed(jwk)) {
                    return Err(InvalidDPoPProofHeader::InvalidJwk(error));
                }
            } else {
                return Err(InvalidDPoPProofHeader::RequiredParamIsAbsent(
                    "jwk".to_owned(),
                ));
            }

            Ok(())
        }
    }

    impl<H: Borrow<JwsHeader>> PurePredicate<H> for IsValidDPoPProofHeader {}
}
