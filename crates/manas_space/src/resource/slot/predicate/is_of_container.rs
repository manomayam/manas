use std::{borrow::Borrow, marker::PhantomData};

use gdp_rs::{
    binclassified::BinaryClassPredicate,
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};

use super::ResourceKindBasedClassification;
use crate::{resource::slot::SolidResourceSlot, SolidStorageSpace};

/// A [`Predicate`] over subject [`SolidResourceSlot`] asserting
/// that slot is of a container resource.
pub struct IsOfContainer<Space> {
    _phantom: PhantomData<Space>,
}

impl<Space> std::fmt::Debug for IsOfContainer<Space> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsOfContainer").finish()
    }
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> Predicate<WS>
    for IsOfContainer<Space>
{
    fn label() -> std::borrow::Cow<'static, str> {
        "IsOfContainer".into()
    }
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> SyncEvaluablePredicate<WS>
    for IsOfContainer<Space>
{
    type EvalError = IsNotAContainerSlot;

    #[inline]
    fn evaluate_for(sub: &WS) -> Result<(), Self::EvalError> {
        let slot = sub.borrow();
        if slot.is_container_slot() {
            Ok(())
        } else {
            Err(IsNotAContainerSlot)
        }
    }
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> PurePredicate<WS>
    for IsOfContainer<Space>
{
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> BinaryClassPredicate<WS>
    for IsOfContainer<Space>
{
    type BinClassification = ResourceKindBasedClassification<WS, Space>;
}

#[derive(Debug, thiserror::Error)]
#[error("Given resource slot is not a container resource slot.")]
/// Error of a slot being not a container slot.
pub struct IsNotAContainerSlot;
