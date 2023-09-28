//! I define [`Conditional`] predicate.

use std::{borrow::Cow, marker::PhantomData};

use crate::{
    predicate::{Predicate, PurePredicate},
    proven::Proven,
};

/// A struct representing a conditional predicate
/// that is conditional over another predicate.
#[allow(clippy::type_complexity)]
pub struct Conditional<S, C, A> {
    _phantom: PhantomData<fn(S) -> Result<(), (C, A)>>,
}

impl<S, C, A> PurePredicate<S> for Conditional<S, C, A>
where
    C: PurePredicate<S>,
    A: PurePredicate<S>,
{
}

impl<S, C, A> Clone for Conditional<S, C, A> {
    fn clone(&self) -> Self {
        Self {
            _phantom: self._phantom,
        }
    }
}

impl<S, C, A> std::fmt::Debug for Conditional<S, C, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllOf").finish()
    }
}

impl<S, C, A> Predicate<S> for Conditional<S, C, A>
where
    C: Predicate<S>,
    A: Predicate<S>,
{
    fn label() -> Cow<'static, str> {
        Cow::Owned(format!("{}|{}", A::label(), C::label()))
    }
}

impl<S, C, A> Proven<S, Conditional<S, C, A>>
where
    C: Predicate<S>,
    A: Predicate<S>,
{
    /// Conditionalize already proven predicate.
    //
    // Safety: If consequence is always true, then it must be true if CPA is true.
    #[inline]
    pub fn from_proven(v: Proven<S, C>) -> Self {
        unsafe { Self::new_unchecked(v.into_subject()) }
    }

    /// Promise that condition in conditional predicate holds,
    /// and convert to consequence proven proposition.
    ///
    /// # Safety
    ///
    /// Callers must promise that CPA (conditional proof assumption) holds.
    #[inline]
    pub unsafe fn promise_condition(self) -> Proven<S, C> {
        Proven::new_unchecked(self.into_subject())
    }
}
