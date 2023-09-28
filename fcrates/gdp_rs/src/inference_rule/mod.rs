//! I define types for representing inference rules.
//!

use std::marker::PhantomData;

use crate::predicate::Predicate;

/// A trait for defining an inference rule.
pub trait InferenceRule {
    /// Type of source proposition's subject.
    type SourceSub;

    /// Type of source proposition's predicate.
    type SourcePredicate: Predicate<Self::SourceSub>;

    /// Type of target proposition's subject.
    type TargetSub;

    /// Type of target proposition's predicate.
    type TargetPredicate: Predicate<Self::TargetSub>;

    /// Type of subject transform.
    type SubjectTransform: Operation<Arg = Self::SourceSub, Result = Self::TargetSub>;
}

/// A trait for representing an operation over a subject.
/// It is analogous to `FnOnce<(Arg, )>`
pub trait Operation {
    /// Type of source proposition's subject.
    type Arg;

    /// Type of target proposition's subject.
    type Result;

    /// Call the operation.
    fn call(self, source_sub: Self::Arg) -> Self::Result;
}

/// A marker trait for marking an inference rule as authorized.
/// This can be used to limit who can authorize an inference, using rust's orphan rules.
pub trait AuthorizedInferenceRuleGhost<TPredicate: Predicate<TSub>, TSub> {}

/// A marker trait for marking a subject transform as predicate preserving transform.
pub trait PreservingTransformGhost<SPredicate: Predicate<Sub>, Sub> {}

/// A struct for representing identity transform.
#[derive(Debug)]
pub struct IdentityTransform<Sub> {
    _phantom: PhantomData<fn() -> Sub>,
}

impl<Sub> Default for IdentityTransform<Sub> {
    #[inline]
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Sub> Operation for IdentityTransform<Sub> {
    type Arg = Sub;
    type Result = Sub;

    #[inline]
    fn call(self, source_sub: Self::Arg) -> Self::Result {
        source_sub
    }
}

impl<Sub, P: Predicate<Sub>> PreservingTransformGhost<P, Sub>
    for PhantomData<IdentityTransform<Sub>>
{
}
