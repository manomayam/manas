//! I define few invariants over [`HttpUri`].
//!

use frunk_core::HList;
use gdp_rs::{predicate::impl_::all_of::AllOf, Proven};

use super::predicate::is_normal::IsNormal;
use crate::{
    predicate::{
        has_no_query::HasNoQuery, has_trailing_slash::PathHasTrailingSlash,
        is_absolute::IsAbsolute, is_secure::IsSecure,
    },
    HttpUri,
};

/// Type alias for an invariant of [`HttpUri`] that is proven to
/// be absolute.
pub type AbsoluteHttpUri = Proven<HttpUri, IsAbsolute>;

/// Type alias for an invariant of [`HttpUri`] that is proven to
/// be normal.
pub type NormalHttpUri = Proven<HttpUri, IsNormal>;

/// Type alias for an invariant of [`HttpUri`] that is proven to
/// be absolute, and normal.
pub type NormalAbsoluteHttpUri = Proven<HttpUri, AllOf<HttpUri, HList!(IsNormal, IsAbsolute)>>;

/// Type alias for an invariant of [`HttpUri`] that is proven to
/// be absolute, normal, hierarchical, and has a trailing slash
/// in it's path..
pub type HierarchicalTrailingSlashHttpUri =
    Proven<HttpUri, AllOf<HttpUri, HList!(IsNormal, IsAbsolute, HasNoQuery, PathHasTrailingSlash)>>;

/// Type alias for an invariant of [`HttpUri`] that is proven to
/// be secure as per given secure transport policy.
pub type SecureHttpUri<STP> = Proven<HttpUri, IsSecure<STP>>;
