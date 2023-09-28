//! I define types for representing predicates and related.
//!

pub mod impl_;
pub mod list;

use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    future::Future,
};

use crate::{BoxError, Proven};

/// Trait for predicates about a subject.
pub trait Predicate<Subject>: Send + Sync + Debug {
    /// Label of the predicate.
    fn label() -> Cow<'static, str>;
}

/// A trait for sync evaluable predicates.
pub trait SyncEvaluablePredicate<S>: Predicate<S> {
    /// Type of evaluation error.
    type EvalError: Into<BoxError> + Send + 'static + Debug + Display;

    /// Evaluate the predicate for given subject.
    fn evaluate_for(sub: &S) -> Result<(), Self::EvalError>;
}

/// A trait for async evaluable predicates.
pub trait AsyncEvaluablePredicate<S>: Predicate<S> {
    /// Type of evaluation error.
    type EvalError: Into<BoxError> + Send + 'static + Debug + Display;

    /// Future for evaluation result.
    type EvalFuture<'s>: Future<Output = Result<(), Self::EvalError>> + Send + 's
    where
        S: 's;

    /// Evaluate the predicate for given subject.
    fn evaluate_for(sub: &S) -> Self::EvalFuture<'_>;
}

/// A trait or marking a [`Predicate`] as pure.
/// A pure predicate evaluation depends upon value of the subject only.
/// Thus, given a subject, a pure predicate always evaluates to same truth value.
pub trait PurePredicate<Subject>: Predicate<Subject> {}

/// Trait for marking a predicate that it will be always be
/// true against any of the subject, and leads to truism.
pub trait TruismPredicate<Subject>: Predicate<Subject> {}

impl<S, TP: TruismPredicate<S>> Proven<S, TP> {
    /// Create a new truism about given subject.
    pub fn new_truism(sub: S) -> Self {
        // Safety: Trueism.
        unsafe { Self::new_unchecked(sub) }
    }
}
