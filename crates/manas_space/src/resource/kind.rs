//! I define [`SolidResourceKind`].

use std::fmt::{Debug, Display};

//ANCHOR: kind-def

/// An enum representing kind of resources in a solid storage space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolidResourceKind {
    /// Container resource kind.
    Container,

    /// Non container resource kind.
    NonContainer,
}

//ANCHOR_END: kind-def

impl Display for SolidResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl SolidResourceKind {
    /// Slice of all resource kinds.
    pub const ALL: &'static [Self] = &[Self::Container, Self::NonContainer];
}
