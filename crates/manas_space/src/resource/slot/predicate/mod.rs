//! I define few predicates over [`SolidResourceSlot`](super::SolidResourceSlot).
//!

use std::{borrow::Borrow, marker::PhantomData};

use gdp_rs::binclassified::BinaryClassification;

use super::SolidResourceSlot;
use crate::SolidStorageSpace;

mod is_of_container;
mod is_of_non_container;

pub use self::{is_of_container::IsOfContainer, is_of_non_container::IsOfNonContainer};

/// An implementation of [`BinaryClassification`] over
/// resource slots based on kind of the slotted resource.
pub struct ResourceKindBasedClassification<WS, Space> {
    _phantom: PhantomData<fn(WS, Space)>,
}

impl<WS: Borrow<SolidResourceSlot<Space>>, Space: SolidStorageSpace> BinaryClassification<WS>
    for ResourceKindBasedClassification<WS, Space>
{
    type LeftPredicate = IsOfContainer<Space>;

    type RightPredicate = IsOfNonContainer<Space>;
}
