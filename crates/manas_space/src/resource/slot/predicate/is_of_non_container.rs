use std::{borrow::Borrow, marker::PhantomData};

use gdp_rs::{
    binclassified::BinaryClassPredicate,
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};

use super::ResourceKindBasedClassification;
use crate::{resource::slot::SolidResourceSlot, SolidStorageSpace};

/// A [`Predicate`] over subject [`SolidResourceSlot`] asserting
/// that slot is of a non-container resource.
pub struct IsOfNonContainer<Space> {
    _phantom: PhantomData<Space>,
}

impl<Space> std::fmt::Debug for IsOfNonContainer<Space> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsOfNonContainer").finish()
    }
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> Predicate<WS>
    for IsOfNonContainer<Space>
{
    fn label() -> std::borrow::Cow<'static, str> {
        "IsOfNonContainer".into()
    }
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> SyncEvaluablePredicate<WS>
    for IsOfNonContainer<Space>
{
    type EvalError = IsNotANonContainerSlot;

    #[inline]
    fn evaluate_for(sub: &WS) -> Result<(), Self::EvalError> {
        let slot = sub.borrow();
        if !slot.is_container_slot() {
            Ok(())
        } else {
            Err(IsNotANonContainerSlot)
        }
    }
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> PurePredicate<WS>
    for IsOfNonContainer<Space>
{
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> BinaryClassPredicate<WS>
    for IsOfNonContainer<Space>
{
    type BinClassification = ResourceKindBasedClassification<WS, Space>;
}

#[derive(Debug, thiserror::Error)]
#[error("Given resource slot is not a non-container resource slot.")]
/// Error of a slot being not a non-container slot.
pub struct IsNotANonContainerSlot;
