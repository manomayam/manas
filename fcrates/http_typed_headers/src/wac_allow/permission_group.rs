use std::{ops::Deref, str::FromStr};

use crate::common::field::rules::alpha::Alpha;

/// A type to represent `permission-group` abnf production fro WAC.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PermissionGroup(Alpha);

impl FromStr for PermissionGroup {
    type Err = InvalidEncodedPermissionGroup;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse().map_err(|_| InvalidEncodedPermissionGroup)?))
    }
}

/// Invalid encoded permission group.
#[derive(Debug, thiserror::Error)]
#[error("Invalid encoded permission group.")]
pub struct InvalidEncodedPermissionGroup;

impl Deref for PermissionGroup {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl PermissionGroup {
    /// `user` permission group.
    pub const USER: Self = Self(Alpha::new_small_unchecked("user"));

    /// `public` permission group.
    pub const PUBLIC: Self = Self(Alpha::new_small_unchecked("public"));
}
