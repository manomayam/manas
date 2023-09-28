//! I define few predicates over [`ODRResourceContext`](super::ODRResourceContext).
//!

use std::{borrow::Borrow, marker::PhantomData};

use gdp_rs::binclassified::BinaryClassification;

use super::ODRResourceContext;
use crate::setup::ODRSetup;

mod is_of_container;
mod is_of_non_container;

pub use self::{is_of_container::IsOfContainer, is_of_non_container::IsOfNonContainer};

/// An implementation of [`BinaryClassification`] over
/// resource contexts based on kind of the resource.
pub struct ResourceKindBasedClassification<WC, Setup> {
    _phantom: PhantomData<fn(WC, Setup)>,
}

impl<WC: Borrow<ODRResourceContext<Setup>>, Setup: ODRSetup> BinaryClassification<WC>
    for ResourceKindBasedClassification<WC, Setup>
{
    type LeftPredicate = IsOfContainer<Setup>;

    type RightPredicate = IsOfNonContainer<Setup>;
}
