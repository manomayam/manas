//! I define an implementation of resource operator for
//! unsupported operations.
//!

use std::marker::PhantomData;

/// An implementation of resource operator for
/// unsupported operations.
pub struct UnsupportedOperator<R> {
    _phantom: PhantomData<R>,
}

impl<R> Default for UnsupportedOperator<R> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<R> Clone for UnsupportedOperator<R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _phantom: self._phantom,
        }
    }
}

impl<R> std::fmt::Debug for UnsupportedOperator<R> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnsupportedOperator").finish()
    }
}
