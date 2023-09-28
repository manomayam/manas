//! I define [`IsSecure`] predicate over `HttpUri`.

use std::{
    borrow::{Borrow, Cow},
    marker::PhantomData,
};

use gdp_rs::{
    inference_rule::{AuthorizedInferenceRuleGhost, IdentityTransform, InferenceRule},
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate, TruismPredicate},
};

use crate::{
    security::transport_policy::{SecureTransportPolicy, VoidSTP},
    HttpUri,
};

/// A predicate about an [`HttpUri`] asserting that it is secure as per give secure transport policy.
#[derive(Debug)]
pub struct IsSecure<STP: SecureTransportPolicy> {
    _phantom: PhantomData<fn(STP)>,
}

impl<STP: SecureTransportPolicy, W: Borrow<HttpUri>> Predicate<W> for IsSecure<STP> {
    fn label() -> Cow<'static, str> {
        "IsSecure<..>".into()
    }
}

impl<STP: SecureTransportPolicy, W: Borrow<HttpUri>> PurePredicate<W> for IsSecure<STP> {}

impl<STP: SecureTransportPolicy, W: Borrow<HttpUri>> SyncEvaluablePredicate<W> for IsSecure<STP> {
    type EvalError = NotASecureHttpUri<STP>;

    #[inline]
    fn evaluate_for(sub: &W) -> Result<(), Self::EvalError> {
        match STP::is_secure(sub.borrow()) {
            true => Ok(()),
            false => Err(NotASecureHttpUri::default()),
        }
    }
}

impl<W: Borrow<HttpUri>> TruismPredicate<W> for IsSecure<VoidSTP> {}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of HttpUri being not a secure uri.
#[error("Given http uri is not secure as per secure transport policy.")]
pub struct NotASecureHttpUri<STP> {
    _phantom: PhantomData<STP>,
}

impl<STP> Default for NotASecureHttpUri<STP> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

/// An implementation of [`InferenceRule`] that infers
/// void-stp compliant invariant from any other stp compliant uri invariant.
pub struct ToVoidSTP<SourceSTP, W: Borrow<HttpUri>> {
    _phantom: PhantomData<fn(SourceSTP, W)>,
}

impl<SourceSTP: SecureTransportPolicy, W: Borrow<HttpUri>> InferenceRule
    for ToVoidSTP<SourceSTP, W>
{
    type SourceSub = W;

    type SourcePredicate = IsSecure<SourceSTP>;

    type TargetSub = W;

    type TargetPredicate = IsSecure<VoidSTP>;

    type SubjectTransform = IdentityTransform<W>;
}

impl<SourceSTP: SecureTransportPolicy, W: Borrow<HttpUri>>
    AuthorizedInferenceRuleGhost<IsSecure<VoidSTP>, W> for PhantomData<ToVoidSTP<SourceSTP, W>>
{
}
