//! I define types to represent binary classified values of a subject type.
//!

use std::ops::Deref;

pub use either::Either;

use crate::{
    predicate::{PurePredicate, SyncEvaluablePredicate},
    Proven,
};

/// A trait for defining a binary classification over a subject type.
pub trait BinaryClassification<Subject> {
    /// Type of left class predicate.
    type LeftPredicate: BinaryClassPredicate<Subject, BinClassification = Self>;

    /// Type of right class predicate.
    type RightPredicate: BinaryClassPredicate<Subject, BinClassification = Self>;
}

/// A trait for defining predicate associated with a class in binary classification.
pub trait BinaryClassPredicate<Subject>: PurePredicate<Subject> {
    /// Type of classification, in which this predicate determines a class.
    type BinClassification: BinaryClassification<Subject>;
}

/// An enum for representing binary classified values of a subject type.
pub struct BinaryClassified<Subject, Cln>(
    pub Either<Proven<Subject, Cln::LeftPredicate>, Proven<Subject, Cln::RightPredicate>>,
)
where
    Cln: BinaryClassification<Subject>;

impl<Subject: Clone, Cln> Clone for BinaryClassified<Subject, Cln>
where
    Cln: BinaryClassification<Subject>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Subject: std::fmt::Debug, Cln> std::fmt::Debug for BinaryClassified<Subject, Cln>
where
    Cln: BinaryClassification<Subject>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BinaryClassified").field(&self.0).finish()
    }
}

impl<Subject, Cln> BinaryClassified<Subject, Cln>
where
    Cln: BinaryClassification<Subject>,
{
    /// Get a new binary classified subject value.
    pub fn new(subject: Subject) -> Self
    where
        Cln::LeftPredicate: SyncEvaluablePredicate<Subject>,
    {
        Self(
            match Proven::<Subject, Cln::LeftPredicate>::try_new(subject) {
                Ok(left_proven) => Either::Left(left_proven),

                // # Safety:
                //
                // Safe, as PL and PR are binary opposite predicates.
                Err(err) => unsafe { Either::Right(Proven::new_unchecked(err.into_parts().0)) },
            },
        )
    }

    /// Get a reference to inner subject value.
    pub fn as_inner(&self) -> &Subject {
        match &self.0 {
            Either::Left(l) => l.as_ref(),
            Either::Right(r) => r.as_ref(),
        }
    }

    /// Convert into inner subject value.
    pub fn into_inner(self) -> Subject {
        match self.0 {
            Either::Left(l) => l.into_subject(),
            Either::Right(r) => r.into_subject(),
        }
    }

    /// Get if inner value is left classified.
    #[inline]
    pub fn is_left_classified(&self) -> bool {
        self.0.is_left()
    }

    /// Get if inner value is left classified.
    #[inline]
    pub fn is_right_classified(&self) -> bool {
        self.0.is_right()
    }

    /// Get as left classified value.
    pub fn as_left_classified(&self) -> Option<&Proven<Subject, Cln::LeftPredicate>> {
        match &self.0 {
            Either::Left(left) => Some(left),
            Either::Right(_) => None,
        }
    }

    /// Get as right classified value.
    pub fn as_right_classified(&self) -> Option<&Proven<Subject, Cln::RightPredicate>> {
        match &self.0 {
            Either::Left(_) => None,
            Either::Right(right) => Some(right),
        }
    }
}

impl<Subject, Cln> Deref for BinaryClassified<Subject, Cln>
where
    Cln: BinaryClassification<Subject>,
{
    type Target = Subject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

impl<Subject, Cln> AsRef<Subject> for BinaryClassified<Subject, Cln>
where
    Cln: BinaryClassification<Subject>,
{
    #[inline]
    fn as_ref(&self) -> &Subject {
        self.as_inner()
    }
}
