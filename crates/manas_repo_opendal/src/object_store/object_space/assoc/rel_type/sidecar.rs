//! I define [`SidecarRelType`].
//!

/// An enum of rel types indicating sidecar association from a resource to an odr object..
// TODO A fat node for internal resource metadata.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidecarRelType {
    /// AltContent object.
    AltContent,

    /// AltFatMeta object.
    AltFatMeta,
}

impl SidecarRelType {
    /// A slice of all sidecar rel types.
    pub const ALL: &'static [SidecarRelType] = &[Self::AltFatMeta, Self::AltContent];
}
