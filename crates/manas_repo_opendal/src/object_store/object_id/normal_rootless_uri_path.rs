//! I define struct for [`NormalRootlessUriPath`].

use std::{borrow::Cow, ops::Deref};

/// A struct representing uri path, that is normal and rootless.
/// It guarantees  that inner path is:
///
/// 1. Rootless
/// 2. Valid pct encoded and normalized o be uri path.
/// 3. Has no non trailing empty path segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalRootlessUriPath<'path> {
    /// Path string.
    path: Cow<'path, str>,
}

impl<'path> NormalRootlessUriPath<'path> {
    /// Get a borrowed [`NormalRootlessUriPath`].
    #[inline]
    pub fn to_borrowed(&self) -> NormalRootlessUriPath<'_> {
        NormalRootlessUriPath {
            path: Cow::Borrowed(self.path.as_ref()),
        }
    }

    /// Convert into owned [`NormalRootlessUriPath`].
    #[inline]
    pub fn into_owned(self) -> NormalRootlessUriPath<'static> {
        NormalRootlessUriPath {
            path: Cow::Owned(self.path.into_owned()),
        }
    }

    /// Get new normal root less uri path from given string without checking.
    ///
    /// # Safety
    ///
    /// Caller must make sure that string is normal, rootless uri path.
    #[inline]
    pub unsafe fn new_unchecked(path: Cow<'path, str>) -> Self {
        Self { path }
    }

    /// Check if path can be of namespace.
    #[inline]
    pub fn is_namespace_path(&self) -> bool {
        self.ends_with('/') || self.is_empty()
    }
    // TODO safe constructor.
}

impl<'path> Deref for NormalRootlessUriPath<'path> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.path.as_ref()
    }
}

impl<'path> AsRef<str> for NormalRootlessUriPath<'path> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.path.as_ref()
    }
}
