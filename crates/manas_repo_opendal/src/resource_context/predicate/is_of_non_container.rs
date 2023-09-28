use std::{borrow::Borrow, marker::PhantomData};

use gdp_rs::{
    binclassified::BinaryClassPredicate,
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};
use manas_space::resource::kind::SolidResourceKind;

use super::ResourceKindBasedClassification;
use crate::{resource_context::ODRResourceContext, setup::ODRSetup};

/// A [`Predicate`] over subject [`ODRResourceContext`]
/// asserting that context is that of a non-container
/// resource.
pub struct IsOfNonContainer<Setup> {
    _phantom: PhantomData<Setup>,
}

impl<Setup> std::fmt::Debug for IsOfNonContainer<Setup> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsOfNonContainer").finish()
    }
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> Predicate<WC>
    for IsOfNonContainer<Setup>
{
    fn label() -> std::borrow::Cow<'static, str> {
        "IsOfNonContainer".into()
    }
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> SyncEvaluablePredicate<WC>
    for IsOfNonContainer<Setup>
{
    type EvalError = IsNotOfANonContainer;

    #[inline]
    fn evaluate_for(sub: &WC) -> Result<(), Self::EvalError> {
        let context = sub.borrow();
        if context.kind() == SolidResourceKind::NonContainer {
            Ok(())
        } else {
            Err(IsNotOfANonContainer)
        }
    }
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> PurePredicate<WC>
    for IsOfNonContainer<Setup>
{
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> BinaryClassPredicate<WC>
    for IsOfNonContainer<Setup>
{
    type BinClassification = ResourceKindBasedClassification<WC, Setup>;
}

#[derive(Debug, thiserror::Error)]
#[error("Given resource context is not that of a container resource.")]
/// Given resource context is not that of a container resource
pub struct IsNotOfANonContainer;
