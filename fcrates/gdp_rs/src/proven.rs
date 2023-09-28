//! I define [`Proven`] struct, for representing a proven
//! proposition about a subject.
//!

use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
    str::FromStr,
};

use frunk_core::HList;

use crate::{
    inference_rule::{
        AuthorizedInferenceRuleGhost, InferenceRule, Operation, PreservingTransformGhost,
    },
    predicate::{
        impl_::all_of::AllOf, AsyncEvaluablePredicate, Predicate, PurePredicate,
        SyncEvaluablePredicate,
    },
};

/// A struct representing a proven proposition about a subject.
pub struct Proven<S, P> {
    /// Subject value.
    subject: S,

    /// Ghost of departed proof.
    _gdp: PhantomData<P>,
}

#[cfg(feature = "serde")]
impl<Sub, P> serde::Serialize for Proven<Sub, P>
where
    Sub: serde::Serialize,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.subject.serialize(serializer)
    }
}

/// An implementation of [`serde::de::Expected`] for indicating
/// proven value expectation.
#[cfg(feature = "serde")]
pub struct ExpectedProven<S, P> {
    _phantom: PhantomData<fn() -> (S, P)>,
}

#[cfg(feature = "serde")]
impl<S, P: Predicate<S>> serde::de::Expected for ExpectedProven<S, P> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a subject value that satisfies {} predicate evaluation",
            P::label().as_ref()
        )
    }
}

#[cfg(feature = "serde")]
impl<'de, Sub, P> serde::Deserialize<'de> for Proven<Sub, P>
where
    Sub: serde::Deserialize<'de>,
    P: SyncEvaluablePredicate<Sub>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let subject = Sub::deserialize(deserializer)?;
        Proven::try_new(subject).map_err(|_| {
            <D::Error as serde::de::Error>::invalid_value(
                serde::de::Unexpected::Other("predicate evaluation failing subject"),
                &ExpectedProven::<Sub, P> {
                    _phantom: PhantomData,
                },
            )
        })
    }
}

impl<S, P> Hash for Proven<S, P>
where
    S: Hash,
    P: PurePredicate<S>,
{
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.subject.hash(state);
    }
}

impl<S, P> Ord for Proven<S, P>
where
    S: Ord,
    P: PurePredicate<S>,
{
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.subject.cmp(&other.subject)
    }
}

impl<S, P> PartialOrd for Proven<S, P>
where
    S: PartialOrd,
    P: PurePredicate<S>,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.subject.partial_cmp(&other.subject)
    }
}

impl<S, P> Eq for Proven<S, P>
where
    S: Eq,
    P: PurePredicate<S>,
{
}

impl<S, P> PartialEq for Proven<S, P>
where
    S: PartialEq,
    P: PurePredicate<S>,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.subject == other.subject
    }
}

impl<S, P> Copy for Proven<S, P>
where
    S: Copy,
    P: PurePredicate<S>,
{
}

impl<S, P> Clone for Proven<S, P>
where
    S: Clone,
    P: PurePredicate<S>,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            subject: self.subject.clone(),
            _gdp: PhantomData,
        }
    }
}

impl<S, P> Debug for Proven<S, P>
where
    S: Debug,
    P: Predicate<S>,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Proven<{}>({:?})", P::label(), self.subject)
    }
}

impl<S, P> Display for Proven<S, P>
where
    S: Display,
    P: Predicate<S>,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.subject.fmt(f)
    }
}

impl<S, P> AsRef<S> for Proven<S, P> {
    #[inline]
    fn as_ref(&self) -> &S {
        &self.subject
    }
}

impl<S, P> Deref for Proven<S, P> {
    type Target = S;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.subject
    }
}

impl<S, P> Borrow<S> for Proven<S, P> {
    #[inline]
    fn borrow(&self) -> &S {
        &self.subject
    }
}

impl<S, P> Proven<S, P>
where
    P: Predicate<S>,
{
    /// Convert into proposition subject value.
    #[inline]
    pub fn into_subject(self) -> S {
        self.subject
    }

    /// Convert subject into other type.
    pub fn subject_into<T>(self) -> Proven<T, P>
    where
        P: Predicate<T>,
        S: Into<T>,
    {
        Proven {
            subject: self.subject.into(),
            _gdp: PhantomData,
        }
    }

    /// Get new proven proposition with given subject value, with out checks
    /// .
    /// # Safety
    ///
    /// Callers must ensure that proposition with given subject is true.
    #[inline]
    pub const unsafe fn new_unchecked(subject: S) -> Self {
        Self {
            subject,
            _gdp: PhantomData,
        }
    }

    /// Apply given preserving transform over proposition subject.
    pub fn transform_sub<Op: Operation<Arg = S, Result = S> + PreservingTransformGhost<P, S>>(
        self,
        op: Op,
    ) -> Proven<S, P> {
        Proven {
            subject: op.call(self.subject),
            _gdp: PhantomData,
        }
    }

    /// Infer new proposition from given proven proposition.
    #[inline]
    pub fn infer<R>(self, op: R::SubjectTransform) -> Proven<R::TargetSub, R::TargetPredicate>
    where
        R: InferenceRule<SourceSub = S, SourcePredicate = P>,
        PhantomData<R>: AuthorizedInferenceRuleGhost<R::TargetPredicate, R::TargetSub>,
    {
        Proven {
            subject: op.call(self.subject),
            _gdp: PhantomData,
        }
    }

    /// Infer new proposition from given proven proposition
    /// using preserving inference rule.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn and_infer<R>(
        self,
        op: R::SubjectTransform,
    ) -> Proven<R::TargetSub, AllOf<S, HList!(R::TargetPredicate, P)>>
    where
        R: InferenceRule<SourceSub = S, TargetSub = S, SourcePredicate = P>,
        PhantomData<R::SubjectTransform>: PreservingTransformGhost<P, S>,
        PhantomData<R>: AuthorizedInferenceRuleGhost<R::TargetPredicate, S>,
    {
        Proven {
            subject: op.call(self.subject),
            _gdp: PhantomData,
        }
    }
}

impl<S, P> Proven<S, P>
where
    P: Predicate<S>,
{
    /// Try to create new proven proposition with given subject value.
    pub fn try_new(subject: S) -> Result<Self, ProvenError<S, P::EvalError>>
    where
        P: SyncEvaluablePredicate<S>,
    {
        match P::evaluate_for(&subject) {
            Ok(_) => Ok(Self {
                subject,
                _gdp: PhantomData,
            }),
            Err(e) => Err(ProvenError { error: e, subject }),
        }
    }

    /// Try to create new proven proposition with given subject value.
    pub async fn try_new_async(subject: S) -> Result<Self, ProvenError<S, P::EvalError>>
    where
        P: AsyncEvaluablePredicate<S>,
    {
        match P::evaluate_for(&subject).await {
            Ok(_) => Ok(Self {
                subject,
                _gdp: PhantomData,
            }),
            Err(e) => Err(ProvenError { error: e, subject }),
        }
    }

    /// Try to create proven proposition about subject that can be converted from given value.
    #[allow(clippy::type_complexity)]
    pub fn try_new_from<V>(
        subject: V,
    ) -> Result<Self, ConvertOrProvenError<<S as TryFrom<V>>::Error, S, P::EvalError>>
    where
        S: TryFrom<V> + TryProven<P, Err = ProvenError<S, P::EvalError>> + Sized,
        P: SyncEvaluablePredicate<S>,
    {
        TryInto::<S>::try_into(subject)
            .map_err(ConvertOrProvenError::ConvertError)?
            .try_proven()
            .map_err(ConvertOrProvenError::ProvenError)
    }
}

/// A trait for getting converting into proven invariant.
pub trait TryProven<P>: Sized {
    /// Type of error.
    type Err: Debug;

    /// A trait for subjects, to try to create proven propositions with evaluating predicate..
    fn try_proven(self) -> Result<Proven<Self, P>, Self::Err>;
}

impl<S, P> TryProven<P> for S
where
    S: Sized,
    P: Predicate<S> + SyncEvaluablePredicate<S>,
{
    type Err = ProvenError<S, P::EvalError>;

    #[inline]
    fn try_proven(self) -> Result<Proven<Self, P>, Self::Err> {
        Proven::try_new(self)
    }
}

impl<S, P> FromStr for Proven<S, P>
where
    S: FromStr,
    P: Predicate<S> + SyncEvaluablePredicate<S>,
{
    type Err = ConvertOrProvenError<S::Err, S, P::EvalError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        S::from_str(s)
            .map_err(ConvertOrProvenError::ConvertError)?
            .try_proven()
            .map_err(ConvertOrProvenError::ProvenError)
    }
}

/// A type representing proven predicate errors about a subject.
#[derive(thiserror::Error)]
#[error("Error in proving predicate about subject. Error: {error}")]
pub struct ProvenError<S, PErr> {
    /// Subject.
    pub subject: S,

    /// Error.
    pub error: PErr,
}

impl<S, PErr: Debug> Debug for ProvenError<S, PErr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProvenError")
            .field("error", &self.error)
            // TODO add subject details?
            .finish()
    }
}

impl<S, PErr> ProvenError<S, PErr> {
    /// Convert into parts.
    #[inline]
    pub fn into_parts(self) -> (S, PErr) {
        (self.subject, self.error)
    }

    /// Get the subject.
    #[inline]
    pub fn subject(&self) -> &S {
        &self.subject
    }

    /// Get the proven error.
    #[inline]
    pub fn error(&self) -> &PErr {
        &self.error
    }
}
/// A sum type representing error for converting + predicate
/// evaluation operation.
#[derive(thiserror::Error)]
pub enum ConvertOrProvenError<C, S, PErr: Display> {
    /// Error in subject type conversion.
    #[error("Error in conversion. Error: {0}")]
    ConvertError(C),

    /// Error in predicate evaluation.
    #[error("Error in proving the predicate about converted subject. Error: {0}")]
    ProvenError(ProvenError<S, PErr>),
}

impl<C, S, PErr: Debug + Display> Debug for ConvertOrProvenError<C, S, PErr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConvertError(_) => f.debug_tuple("ConvertError").finish(),
            Self::ProvenError(e) => f.debug_tuple("ProvenError").field(e).finish(),
        }
    }
}
