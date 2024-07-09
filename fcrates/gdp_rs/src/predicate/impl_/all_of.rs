//! I define [`AllOf`] compound predicate.

use std::{borrow::Cow, marker::PhantomData};

pub use frunk_core::{
    hlist::{HCons, HNil},
    HList,
};

use frunk_core::{
    hlist::{Plucker, Sculptor},
    traits::IntoReverse,
};

use crate::{
    inference_rule::{AuthorizedInferenceRuleGhost, IdentityTransform, InferenceRule},
    predicate::{
        list::{
            AsyncEvaluablePredicateList, PredicateList, PurePredicateList,
            SyncEvaluablePredicateList,
        },
        AsyncEvaluablePredicate, Predicate, PurePredicate, SyncEvaluablePredicate,
    },
    proven::Proven,
    BoxError, BoxFuture,
};

/// A struct representing a compound predicate
/// that evaluates to logical conjunction of all list members.
pub struct AllOf<S, PL: PredicateList<S>> {
    _phantom: PhantomData<fn() -> (S, PL)>,
}

impl<S, PL: PredicateList<S>> Clone for AllOf<S, PL> {
    fn clone(&self) -> Self {
        Self {
            _phantom: self._phantom,
        }
    }
}

impl<S, PL: PredicateList<S>> std::fmt::Debug for AllOf<S, PL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllOf").finish()
    }
}

impl<S, PL> Predicate<S> for AllOf<S, PL>
where
    PL: PredicateList<S>,
{
    #[inline]
    fn label() -> Cow<'static, str> {
        let mut buf = String::from("AllOf");
        buf.push('(');
        PL::fmt_joined_label(&mut buf, ", ");
        buf.push(')');
        Cow::Owned(buf)
    }
}

impl<S, PL: PurePredicateList<S>> PurePredicate<S> for AllOf<S, PL> {}

impl<S, PL> Proven<S, AllOf<S, PL>>
where
    PL: PredicateList<S>,
{
    /// Try to extend all-of compound predicate with predicate of given predicate class about subject.
    #[allow(clippy::type_complexity)]
    pub fn try_extend_predicate<P>(
        self,
    ) -> Result<Proven<S, AllOf<S, HCons<P, PL>>>, <P as SyncEvaluablePredicate<S>>::EvalError>
    where
        P: Predicate<S> + SyncEvaluablePredicate<S>,
    {
        P::evaluate_for(self.as_ref())?;
        // Safety: All predicates hold.
        unsafe { Ok(Proven::new_unchecked(self.into_subject())) }
    }

    /// Try to extend all-of compound predicate with predicate of given predicate class about subject.
    #[allow(clippy::type_complexity)]
    pub async fn async_try_extend_predicate<P>(
        self,
    ) -> Result<Proven<S, AllOf<S, HCons<P, PL>>>, <P as AsyncEvaluablePredicate<S>>::EvalError>
    where
        P: Predicate<S> + AsyncEvaluablePredicate<S>,
    {
        P::evaluate_for(self.as_ref()).await?;
        // Safety: All predicates hold.
        unsafe { Ok(Proven::new_unchecked(self.into_subject())) }
    }

    /// Extend all-of compound predicate with predicate of given predicate class about subject.
    ///
    /// # Safety
    ///
    /// Callers must ensure, extension predicate about the subject holds.
    pub unsafe fn extend_predicate_unchecked<P>(self) -> Proven<S, AllOf<S, HCons<P, PL>>>
    where
        P: Predicate<S>,
    {
        Proven::new_unchecked(self.into_subject())
    }
}

impl<S> Proven<S, AllOf<S, HNil>> {
    /// Get new proven predicate with given subject and empty list predicate type.
    pub fn new(subject: S) -> Self {
        // Safety: Conjunction of empty list of predicate always holds.
        unsafe { Self::new_unchecked(subject) }
    }
}

impl<S, PL> SyncEvaluablePredicate<S> for AllOf<S, PL>
where
    PL: SyncEvaluablePredicateList<S>,
{
    type EvalError = BoxError;

    #[inline]
    fn evaluate_for(sub: &S) -> Result<(), Self::EvalError> {
        PL::evaluate_all_for(sub)
    }
}

impl<S, PL> AsyncEvaluablePredicate<S> for AllOf<S, PL>
where
    PL: AsyncEvaluablePredicateList<S>,
{
    type EvalError = BoxError;

    type EvalFuture<'s> = BoxFuture<'s, Result<(), BoxError>> where S: 's;

    #[inline]
    fn evaluate_for(sub: &S) -> Self::EvalFuture<'_> {
        PL::evaluate_all_for(sub)
    }
}

/// An operation over proposition with `AllOf`
/// predicate,that sculpts proof predicate list.
#[allow(clippy::type_complexity)]
pub struct SculptPL<Sub, SPL, TPL, Indices> {
    _phantom: PhantomData<fn(Sub, SPL) -> (Sub, (TPL, Indices))>,
}

impl<Sub, SPL, TPL, Indices> InferenceRule for SculptPL<Sub, SPL, TPL, Indices>
where
    SPL: PredicateList<Sub> + Sculptor<TPL, Indices>,
    TPL: PredicateList<Sub>,
{
    type SourceSub = Sub;

    type SourcePredicate = AllOf<Sub, SPL>;

    type TargetSub = Sub;

    type TargetPredicate = AllOf<Sub, TPL>;

    type SubjectTransform = IdentityTransform<Sub>;
}

impl<Sub, SPL, TPL, Indices> AuthorizedInferenceRuleGhost<AllOf<Sub, TPL>, Sub>
    for PhantomData<SculptPL<Sub, SPL, TPL, Indices>>
where
    SPL: PredicateList<Sub> + Sculptor<TPL, Indices>,
    TPL: PredicateList<Sub>,
{
}

/// An operation over proposition with `AllOf`
/// predicate, that plucks a proven predicate from predicate list.
#[allow(clippy::type_complexity)]
pub struct PluckPL<Sub, SPL, TP, Index> {
    _phantom: PhantomData<fn(Sub, SPL) -> (Sub, (TP, Index))>,
}

impl<Sub, SPL, TP, Index> InferenceRule for PluckPL<Sub, SPL, TP, Index>
where
    SPL: PredicateList<Sub> + Plucker<TP, Index>,
    TP: Predicate<Sub>,
{
    type SourceSub = Sub;

    type SourcePredicate = AllOf<Sub, SPL>;

    type TargetSub = Sub;

    type TargetPredicate = TP;

    type SubjectTransform = IdentityTransform<Sub>;
}

impl<Sub, SPL, TP, Index> AuthorizedInferenceRuleGhost<AllOf<Sub, TP>, Sub>
    for PhantomData<PluckPL<Sub, SPL, TP, Index>>
where
    SPL: PredicateList<Sub> + Plucker<TP, Index>,
    TP: PredicateList<Sub>,
{
}

/// An operation over proposition with `AllOf`
/// predicate, that reverses proven predicate list.
pub struct ReversePL<Sub, SPL> {
    _phantom: PhantomData<fn(Sub, SPL)>,
}

impl<Sub, SPL> InferenceRule for ReversePL<Sub, SPL>
where
    SPL: PredicateList<Sub> + IntoReverse,
    <SPL as IntoReverse>::Output: PredicateList<Sub>,
{
    type SourceSub = Sub;

    type SourcePredicate = AllOf<Sub, SPL>;

    type TargetSub = Sub;

    type TargetPredicate = AllOf<Sub, <SPL as IntoReverse>::Output>;

    type SubjectTransform = IdentityTransform<Sub>;
}

impl<Sub, SPL> AuthorizedInferenceRuleGhost<AllOf<Sub, <SPL as IntoReverse>::Output>, Sub>
    for PhantomData<ReversePL<Sub, SPL>>
where
    SPL: PredicateList<Sub> + IntoReverse,
    <SPL as IntoReverse>::Output: PredicateList<Sub>,
{
}

/// An operation over proposition with any
/// predicate, that transforms to proposition with `AllOf` predicate.
pub struct IntoPL<Sub, SP> {
    _phantom: PhantomData<fn(Sub, SP)>,
}

impl<Sub, SP> InferenceRule for IntoPL<Sub, SP>
where
    SP: Predicate<Sub>,
{
    type SourceSub = Sub;

    type SourcePredicate = SP;

    type TargetSub = Sub;

    type TargetPredicate = AllOf<Sub, HList!(SP)>;

    type SubjectTransform = IdentityTransform<Sub>;
}

impl<Sub, SP> AuthorizedInferenceRuleGhost<AllOf<Sub, HList!(SP)>, Sub>
    for PhantomData<IntoPL<Sub, SP>>
where
    SP: Predicate<Sub>,
{
}
