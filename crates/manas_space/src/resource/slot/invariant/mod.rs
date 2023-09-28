//! I define few invariants of [`SolidResourceSlot`](super::SolidResourceSlot).
//!

use gdp_rs::{binclassified::BinaryClassified, Proven};

use super::predicate::{IsOfContainer, IsOfNonContainer, ResourceKindBasedClassification};

/// A type alias for wrapper of a [`SolidResourceSlot`](super::SolidResourceSlot),
/// ensuring inner slot is that of a container.
pub type ContainerSlot<WS, S> = Proven<WS, IsOfContainer<S>>;

/// A type alias for wrapper of a [`SolidResourceSlot`](super::SolidResourceSlot),
/// ensuring inner slot is that of a non container.
pub type NonContainerSlot<WS, S> = Proven<WS, IsOfNonContainer<S>>;

/// A type alias for classified invariant of [`SolidResourceSlot`](super::SolidResourceSlot).
pub type ClassifiedResourceSlot<WS, S> =
    BinaryClassified<WS, ResourceKindBasedClassification<WS, S>>;
