//! This crate provides types for size capped streams.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::{
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use pin_project_lite::pin_project;

/// A trait for item weighers.
pub trait ItemWeigher {
    /// Type of items.
    type Item;

    /// Weigh the item.
    fn weigh(&self, item: &Self::Item) -> u64;
}

/// An [`ItemWeigher`] that weighs all items to unit.
pub struct UnitWeigher<I>(PhantomData<I>);

impl<I> Default for UnitWeigher<I> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<I> Copy for UnitWeigher<I> {}

impl<I> Clone for UnitWeigher<I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I> std::fmt::Debug for UnitWeigher<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("UnitWeigher").finish()
    }
}

impl<I> ItemWeigher for UnitWeigher<I> {
    type Item = I;

    #[inline]
    fn weigh(&self, _item: &Self::Item) -> u64 {
        1
    }
}

/// An [`ItemWeigher`] that weighs byte chunks to their length.
pub struct BytesWeigher<C>(PhantomData<C>);

impl<C> Default for BytesWeigher<C> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<C> Copy for BytesWeigher<C> {}

impl<C> Clone for BytesWeigher<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> std::fmt::Debug for BytesWeigher<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BytesWeigher").finish()
    }
}

impl<C: Deref<Target = [u8]>> ItemWeigher for BytesWeigher<C> {
    type Item = C;

    #[inline]
    fn weigh(&self, item: &Self::Item) -> u64 {
        item.len() as u64
    }
}

pin_project! {
    /// A struct for max size capped data streams.
    #[derive(Debug, Clone)]
    pub struct CappedStream<ItemOk, ItemErr, Inner, Weigher> {
        #[pin]
        inner: Inner,
        weigher: Weigher,
        size_limit: u64,
        consumed: u64,
        _phantom: PhantomData<fn(ItemOk, ItemErr)>,
    }
}

impl<ItemOk, ItemErr, Inner, Weigher> CappedStream<ItemOk, ItemErr, Inner, Weigher> {
    /// Create a new [`CappedStream`] wrapping given inner stream.
    #[inline]
    pub fn new(inner: Inner, weigher: Weigher, size_limit: u64) -> Self {
        Self {
            inner,
            weigher,
            size_limit,
            consumed: 0,
            _phantom: PhantomData,
        }
    }
}

impl<ItemOk, ItemErr, Inner, Weigher> Stream for CappedStream<ItemOk, ItemErr, Inner, Weigher>
where
    Weigher: ItemWeigher<Item = ItemOk>,
    Inner: Stream<Item = Result<ItemOk, ItemErr>>,
    OutOfSizeLimitError: Into<ItemErr>,
{
    type Item = Result<ItemOk, ItemErr>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.inner.poll_next(cx) {
            Poll::Ready(opt_item) => Poll::Ready(opt_item.map(|item| {
                item.and_then(|v| {
                    *this.consumed += this.weigher.weigh(&v);
                    if this.consumed > this.size_limit {
                        Err(OutOfSizeLimitError {
                            limit: *this.size_limit,
                        }
                        .into())
                    } else {
                        Ok(v)
                    }
                })
            })),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Out of size limit.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Out of size limit. Limit was: {limit}")]
pub struct OutOfSizeLimitError {
    /// Size limit.
    pub limit: u64,
}
