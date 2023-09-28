//! I define [`IsNormal`] predicate over `HttpUri`.

use std::marker::PhantomData;

use gdp_rs::{
    inference_rule::{
        AuthorizedInferenceRuleGhost, InferenceRule, Operation, PreservingTransformGhost,
    },
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};

use super::is_absolute::IsAbsolute;
use crate::HttpUri;

/// A predicate about an [`HttpUri`] asserting that it is in normal form.
#[derive(Debug)]
pub struct IsNormal;

impl Predicate<HttpUri> for IsNormal {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsNormal".into()
    }
}

impl PurePredicate<HttpUri> for IsNormal {}

impl SyncEvaluablePredicate<HttpUri> for IsNormal {
    type EvalError = NotANormalHttpUri;

    #[inline]
    fn evaluate_for(sub: &HttpUri) -> Result<(), Self::EvalError> {
        match sub.is_http_normalized() {
            true => Ok(()),
            false => Err(NotANormalHttpUri),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of HttpUri being not a normal uri.
#[error("Given http uri is not in normal form")]
pub struct NotANormalHttpUri;

/// An inference rule, that infers normalized [`HttpUri`] from a given [`HttpUri`].
pub struct Normalization<SP> {
    _phantom: PhantomData<SP>,
}

/// A struct representing normalization transform over a subject [`HttpUri`].
#[derive(Debug, Default)]
pub struct NormalizationTransform;

impl Operation for NormalizationTransform {
    type Arg = HttpUri;

    type Result = HttpUri;

    #[inline]
    fn call(self, source_sub: Self::Arg) -> Self::Result {
        source_sub.http_normalized()
    }
}

impl<SP: Predicate<HttpUri>> InferenceRule for Normalization<SP> {
    type SourceSub = HttpUri;

    type SourcePredicate = SP;

    type TargetSub = HttpUri;

    type TargetPredicate = IsNormal;

    type SubjectTransform = NormalizationTransform;
}

impl<SP: Predicate<HttpUri>> AuthorizedInferenceRuleGhost<IsNormal, HttpUri>
    for PhantomData<Normalization<SP>>
{
}

// Normalization preserves `IsAbsolute` predicate.
impl PreservingTransformGhost<IsAbsolute, HttpUri> for PhantomData<NormalizationTransform> {}
