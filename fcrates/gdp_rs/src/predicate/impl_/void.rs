use std::convert::Infallible;

use crate::{
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate, TruismPredicate},
    Proven,
};

// Implementation of `Predicate` for `()`. This predicate always applies for any subject.
impl<S> Predicate<S> for () {
    fn label() -> std::borrow::Cow<'static, str> {
        "()".into()
    }
}

impl<S> SyncEvaluablePredicate<S> for () {
    type EvalError = Infallible;

    #[inline]
    fn evaluate_for(_sub: &S) -> Result<(), Self::EvalError> {
        Ok(())
    }
}

impl<S> PurePredicate<S> for () {}
impl<S> TruismPredicate<S> for () {}

impl<S> Proven<S, ()> {
    /// Get a `Proven` over given subject and void predicate.
    #[inline]
    pub fn void_proven(sub: S) -> Self {
        Self::new_truism(sub)
    }
}
