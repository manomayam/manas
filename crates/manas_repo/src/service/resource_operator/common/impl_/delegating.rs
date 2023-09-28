//! I define an implementation of resource operator for
//! delegated operations.
//!

use std::{fmt::Debug, marker::PhantomData};

/// An implementation of resource operator for
/// delegated operations.
pub struct DelegatingOperator<Inner, LR> {
    pub(crate) inner: Inner,
    _phantom: PhantomData<fn(LR)>,
}

impl<Inner: Default, LR> Default for DelegatingOperator<Inner, LR> {
    fn default() -> Self {
        Self {
            inner: Inner::default(),
            _phantom: Default::default(),
        }
    }
}

impl<Inner: Clone, LR> Clone for DelegatingOperator<Inner, LR> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<Inner: Debug, LR> Debug for DelegatingOperator<Inner, LR> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatingOperator")
            .field("inner", &self.inner)
            .finish()
    }
}
