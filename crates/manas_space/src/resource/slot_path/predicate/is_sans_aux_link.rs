use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::{resource::slot_path::SolidResourceSlotPath, SolidStorageSpace};

/// A [`Predicate`] over subject [`SolidResourceSlotPath`] asserting
/// that path is sans aux link.
#[derive(Debug)]
pub struct IsSansAuxLink;

impl<'p, Space: SolidStorageSpace> Predicate<SolidResourceSlotPath<'p, Space>> for IsSansAuxLink {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsSansAuxLink".into()
    }
}

impl<'p, Space: SolidStorageSpace> SyncEvaluablePredicate<SolidResourceSlotPath<'p, Space>>
    for IsSansAuxLink
{
    type EvalError = IsNotSansAuxLink;

    #[inline]
    fn evaluate_for(sub: &SolidResourceSlotPath<'p, Space>) -> Result<(), Self::EvalError> {
        if sub.slots().iter().all(|slot| {
            slot.prov_rev_rel_type()
                .map(|rel_type| !rel_type.is_auxiliary())
                .unwrap_or(true)
        }) {
            Ok(())
        } else {
            Err(IsNotSansAuxLink)
        }
    }
}

impl<'p, Space: SolidStorageSpace> PurePredicate<SolidResourceSlotPath<'p, Space>>
    for IsSansAuxLink
{
}

#[derive(Debug, thiserror::Error)]
#[error("Given slot path is not sans aux link.")]
/// Given slot path is not sans aux link.
pub struct IsNotSansAuxLink;
