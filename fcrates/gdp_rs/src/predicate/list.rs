//! I define a [`PredicateList`] and related structs
//! that help in creating compound predicates easier.

use frunk_core::hlist::{HCons, HList, HNil};

use super::{AsyncEvaluablePredicate, Predicate, PurePredicate, SyncEvaluablePredicate};
use crate::{BoxError, BoxFuture};

/// A trait for `HList` of [`Predicate`]s..
pub trait PredicateList<S>: HList {
    /// Jin the labels of predicates and push into buffer.
    fn fmt_joined_label(buf: &mut String, sep: &str);
}

/// A marker trait for list of pure predicates.
pub trait PurePredicateList<S>: PredicateList<S> {}

/// A marker trait for list of evaluable predicates.
pub trait SyncEvaluablePredicateList<S>: PredicateList<S> {
    /// Evaluate the all predicates in list for given subject.
    fn evaluate_all_for(sub: &S) -> Result<(), BoxError>;
}

/// A marker trait for list of evaluable predicates.
pub trait AsyncEvaluablePredicateList<S>: PredicateList<S> {
    /// Evaluate the all predicates in list for given subject.
    fn evaluate_all_for<'s>(sub: &'s S) -> BoxFuture<'s, Result<(), BoxError>>
    where
        S: 's;
}

impl<S, H, T> PredicateList<S> for HCons<H, T>
where
    H: Predicate<S>,
    T: PredicateList<S>,
{
    fn fmt_joined_label(buf: &mut String, sep: &str) {
        buf.push_str(H::label().as_ref());
        buf.push_str(sep);
        T::fmt_joined_label(buf, sep);
    }
}

impl<S, H, T> PurePredicateList<S> for HCons<H, T>
where
    H: PurePredicate<S>,
    T: PurePredicateList<S>,
{
}

impl<S, H, T> SyncEvaluablePredicateList<S> for HCons<H, T>
where
    H: SyncEvaluablePredicate<S>,
    T: SyncEvaluablePredicateList<S>,
{
    fn evaluate_all_for(sub: &S) -> Result<(), BoxError> {
        match H::evaluate_for(sub) {
            Ok(_) => T::evaluate_all_for(sub),
            Err(e) => Err(e.into()),
        }
    }
}

impl<S, H, T> AsyncEvaluablePredicateList<S> for HCons<H, T>
where
    H: AsyncEvaluablePredicate<S>,
    T: AsyncEvaluablePredicateList<S>,
    S: Sync + 'static,
{
    fn evaluate_all_for<'s>(sub: &'s S) -> BoxFuture<'s, Result<(), BoxError>>
    where
        S: 's,
    {
        Box::pin(async move {
            match H::evaluate_for(sub).await {
                Ok(_) => T::evaluate_all_for(sub).await,
                Err(e) => Err(e.into()),
            }
        })
    }
}

impl<S> PredicateList<S> for HNil {
    fn fmt_joined_label(buf: &mut String, sep: &str) {
        // Remove any trailing separator, as list ends.
        if buf.ends_with(sep) {
            buf.truncate(buf.len() - sep.len())
        }
    }
}

impl<S> PurePredicateList<S> for HNil {}

impl<S> SyncEvaluablePredicateList<S> for HNil {
    #[inline]
    fn evaluate_all_for(_sub: &S) -> Result<(), BoxError> {
        Ok(())
    }
}

impl<S> AsyncEvaluablePredicateList<S> for HNil {
    #[inline]
    fn evaluate_all_for<'s>(_sub: &'s S) -> BoxFuture<'s, Result<(), BoxError>>
    where
        S: 's,
    {
        Box::pin(async { Ok(()) })
    }
}
