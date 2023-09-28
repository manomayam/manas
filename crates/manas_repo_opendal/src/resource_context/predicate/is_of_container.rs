use std::{borrow::Borrow, marker::PhantomData};

use gdp_rs::{
    binclassified::BinaryClassPredicate,
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};
use manas_space::resource::kind::SolidResourceKind;

use super::ResourceKindBasedClassification;
use crate::{resource_context::ODRResourceContext, setup::ODRSetup};

/// A [`Predicate`] over subject [`ODRResourceContext`]
/// asserting that context is that of a container resource.
pub struct IsOfContainer<Setup> {
    _phantom: PhantomData<Setup>,
}

impl<Setup> std::fmt::Debug for IsOfContainer<Setup> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsOfContainer").finish()
    }
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> Predicate<WC>
    for IsOfContainer<Setup>
{
    fn label() -> std::borrow::Cow<'static, str> {
        "IsOfContainer".into()
    }
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> SyncEvaluablePredicate<WC>
    for IsOfContainer<Setup>
{
    type EvalError = IsNotOfAContainer;

    #[inline]
    fn evaluate_for(sub: &WC) -> Result<(), Self::EvalError> {
        let context = sub.borrow();
        if context.kind() == SolidResourceKind::Container {
            Ok(())
        } else {
            Err(IsNotOfAContainer)
        }
    }
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> PurePredicate<WC>
    for IsOfContainer<Setup>
{
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> BinaryClassPredicate<WC>
    for IsOfContainer<Setup>
{
    type BinClassification = ResourceKindBasedClassification<WC, Setup>;
}

#[derive(Debug, thiserror::Error)]
#[error("Given resource context is not that of a non-container resource.")]
/// Given resource context is not that of a non-container resource
pub struct IsNotOfAContainer;
