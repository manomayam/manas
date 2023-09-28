//! I define a void implementation of [`NameLocker`].
//!

use std::{hash::Hash, marker::PhantomData};

use futures::{future::BoxFuture, stream::BoxStream};

use crate::{LockKind, NameLocker};

/// An implementation of [`NameLocker`] that doesn't apply any locking.
#[derive(Debug, Default, Clone)]
pub struct VoidNameLocker<Name> {
    _phantom: PhantomData<fn(Name)>,
}

impl<Name> NameLocker for VoidNameLocker<Name>
where
    Name: Ord + Hash + Clone + Send + Sync + 'static,
{
    type Name = Name;

    fn poll_with_lock<Output, Task>(
        &self,
        task: Task,
        _name: Option<Self::Name>,
        _lock_kind: LockKind,
    ) -> BoxFuture<'static, Output>
    where
        Task: futures::Future<Output = Output> + Send + 'static,
    {
        Box::pin(task)
    }

    #[inline]
    fn poll_read_with_lock<'s, S>(
        &self,
        stream: S,
        _name: Option<Self::Name>,
        _lock_kind: LockKind,
    ) -> BoxStream<'s, S::Item>
    where
        S: futures::Stream + Send + 's,
        <S as futures::Stream>::Item: Send,
    {
        Box::pin(stream)
    }
}
