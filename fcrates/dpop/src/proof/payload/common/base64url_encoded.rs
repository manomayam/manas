//! I define types to represent base6url encoded values.
//!

use std::{borrow::Borrow, marker::PhantomData};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use gdp_rs::{
    inference_rule::{AuthorizedInferenceRuleGhost, InferenceRule, Operation},
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex to match a base64url encoded value.
/// @see: <https://base64.guru/standards/base64url>
static BASE64URL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^[A-Za-z0-9_-]+$"#).expect("Must be valid regex"));

/// A predicate over `&str`, asserting that it is a valid base64url encoded value.
#[derive(Debug)]
pub struct IsValidBase64UrlEncodedValue;

impl<T: Borrow<str>> Predicate<T> for IsValidBase64UrlEncodedValue {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsValidBase64UrlEncodedValue".into()
    }
}

/// An error type for invalid base64url values.
#[derive(Debug, thiserror::Error)]
#[error("Invalid base64url encoded value.")]
pub struct InvalidBase64UrlEncodedValue;

impl<T: Borrow<str>> SyncEvaluablePredicate<T> for IsValidBase64UrlEncodedValue {
    type EvalError = InvalidBase64UrlEncodedValue;

    fn evaluate_for(sub: &T) -> Result<(), Self::EvalError> {
        if BASE64URL_RE.is_match(sub.borrow()) {
            Ok(())
        } else {
            Err(InvalidBase64UrlEncodedValue)
        }
    }
}

impl<T: Borrow<str>> PurePredicate<T> for IsValidBase64UrlEncodedValue {}

/// A transform, that encodes given slice of bytes in base64url encoding.
#[derive(Debug, Clone, Default)]
pub struct Base64UrlEncodingTransform<T: AsRef<[u8]>> {
    _phantom: PhantomData<fn(T)>,
}

impl<T: AsRef<[u8]>> Operation for Base64UrlEncodingTransform<T> {
    type Arg = T;

    type Result = String;

    #[inline]
    fn call(self, source_sub: Self::Arg) -> Self::Result {
        URL_SAFE_NO_PAD.encode(source_sub)
    }
}

/// An inference, that states that result of [`Base64UrlEncodingTransform`] will always satisfies [`IsValidBase64UrlEncodedValue`] predicate.
#[derive(Debug, Clone, Default)]
pub struct Base64UrlEncodingRule<T: AsRef<[u8]>> {
    _phantom: PhantomData<fn(T)>,
}

impl<T> InferenceRule for Base64UrlEncodingRule<T>
where
    T: AsRef<[u8]>,
{
    type SourceSub = T;

    type SourcePredicate = ();

    type TargetSub = String;

    type TargetPredicate = IsValidBase64UrlEncodedValue;

    type SubjectTransform = Base64UrlEncodingTransform<T>;
}

impl<T: AsRef<[u8]>> AuthorizedInferenceRuleGhost<IsValidBase64UrlEncodedValue, String>
    for PhantomData<Base64UrlEncodingRule<T>>
{
}
