//! I define few invariants of [`ODRResourceContext`].
//!

use std::sync::Arc;

use gdp_rs::{binclassified::BinaryClassified, Proven};

use super::{
    predicate::{IsOfContainer, IsOfNonContainer, ResourceKindBasedClassification},
    ODRResourceContext,
};

/// A type alias for [`ODRResourceContext`], that is proven
/// to be that of container.
pub type ODRContainerContext<Setup> = Proven<Arc<ODRResourceContext<Setup>>, IsOfContainer<Setup>>;

/// A type alias for [`ODRResourceContext`], that is proven
/// to be that of non-container.
pub type ODRNonContainerContext<Setup> =
    Proven<Arc<ODRResourceContext<Setup>>, IsOfNonContainer<Setup>>;

/// A type alias for classified [`ODRResourceContext`],
pub type ODRClassifiedResourceContext<Setup> = BinaryClassified<
    Arc<ODRResourceContext<Setup>>,
    ResourceKindBasedClassification<Arc<ODRResourceContext<Setup>>, Setup>,
>;
