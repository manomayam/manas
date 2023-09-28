//! I define a predicate over [`SolidResourceState`] that
//! evaluates if resource is represented.
//!

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::{resource::state::SolidResourceState, SolidStorageSpace};

/// A predicate over [`SolidResourceState`] that evaluates if
/// resource is represented.
pub struct IsRepresented {}

impl std::fmt::Debug for IsRepresented {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsRepresented").finish()
    }
}

impl<Space: SolidStorageSpace, Rep> Predicate<SolidResourceState<Space, Rep>> for IsRepresented {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsRepresented".into()
    }
}

impl<Space: SolidStorageSpace, Rep> PurePredicate<SolidResourceState<Space, Rep>>
    for IsRepresented
{
}

impl<Space: SolidStorageSpace, Rep> SyncEvaluablePredicate<SolidResourceState<Space, Rep>>
    for IsRepresented
{
    type EvalError = IsNotRepresentedResourceState;

    fn evaluate_for(sub: &SolidResourceState<Space, Rep>) -> Result<(), Self::EvalError> {
        // Check if representation is some.
        if sub.representation.is_some() {
            Ok(())
        } else {
            Err(IsNotRepresentedResourceState)
        }
    }
}

/// An error type for errors of resource state being not that of
/// a represented resource.
#[derive(Debug, thiserror::Error)]
#[error("Resource state is not that of a represented resource.")]
pub struct IsNotRepresentedResourceState;
