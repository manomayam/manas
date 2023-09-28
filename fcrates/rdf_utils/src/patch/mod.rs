//! I define traits and implementations
//! of few patching algorithms to patch rdf data.
//!

#[cfg(feature = "solid-insert-delete-patch")]
pub mod solid_insert_delete;

/// An enum of operations that can be performed by a patcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatchEffectiveOperation {
    /// Read the resource.
    Read,

    /// Append the resource.
    Append,

    /// Write the resource.
    Write,
}
