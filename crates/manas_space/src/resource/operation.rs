//! I define types for representing operations over resources.
//!

use std::fmt::Display;

/// A struct, describing a solid resource operation.
/// A resource operation can be specialization of another op.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SolidResourceOperation {
    /// Label of the resource operation.
    pub label: &'static str,

    /// Reference to the operation,
    /// to which it is a specialization of.
    pub specialization_of: Option<&'static SolidResourceOperation>,
}

impl Display for SolidResourceOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl SolidResourceOperation {
    /// Create a new [`SolidResourceOperation`].
    #[inline]
    pub const fn new(label: &'static str, specialization_of: Option<&'static Self>) -> Self {
        Self {
            label,
            specialization_of,
        }
    }

    /// Get generalized resource operations of this operation.
    #[inline]
    pub fn generalized(&self) -> impl Iterator<Item = Self> {
        GeneralizedResourceOps {
            current_op: Some(*self),
        }
    }

    /// Read resource operation.
    pub const READ: Self = Self::new("READ", None);

    /// Write resource operation.
    pub const WRITE: Self = Self::new("WRITE", None);

    /// Append resource operation.
    pub const APPEND: Self = Self::new("APPEND", Some(&Self::WRITE));

    // /// Update resource operation.
    // pub const UPDATE: Self = Self::new("UPDATE", Some(&Self::WRITE));

    /// Create resource operation.
    pub const CREATE: Self = Self::new("CREATE", Some(&Self::WRITE));

    /// Delete resource operation.
    pub const DELETE: Self = Self::new("DELETE", Some(&Self::WRITE));
}

/// Iterate over generalized resource ops of a given resource op.
/// Every resource op is considered generalized version of itself.
pub struct GeneralizedResourceOps {
    /// Current operation.
    pub current_op: Option<SolidResourceOperation>,
}

impl Iterator for GeneralizedResourceOps {
    type Item = SolidResourceOperation;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(op) = self.current_op {
            // Update cursor to generalized.
            self.current_op = op.specialization_of.copied();
            // Return current.
            Some(op)
        } else {
            None
        }
    }
}
