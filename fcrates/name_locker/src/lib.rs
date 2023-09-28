//! This crate defines trait for asynchronous name lockers,
//! that can run an async task with advisory-lock on a given name.
//!
//! It also provides a default inmemory implementation
//! using dashmap.
//!

#![warn(missing_docs)]
#![deny(unused_qualifications)]

use std::hash::Hash;

use futures::{future::BoxFuture, stream::BoxStream, Future, Stream};

pub mod impl_;

/// An enum for kind of lock.
#[derive(Debug, Clone)]
pub enum LockKind {
    /// Shared lock.
    Shared,

    /// Exclusive lock.
    Exclusive,
}

/// A trait for name locker.
pub trait NameLocker: Send + Sync + 'static {
    /// Type of the names.
    type Name: Ord + Hash + Send + Sync + 'static;

    /// Create a wrapper task that polls given task with specified locking on specified name.
    fn poll_with_lock<Output, Task>(
        &self,
        task: Task,
        name: Option<Self::Name>,
        lock_kind: LockKind,
    ) -> BoxFuture<'static, Output>
    where
        Task: Future<Output = Output> + Send + 'static;

    /// Create a wrapper stream, that wraps given stream with specified locking on specified name.
    fn poll_read_with_lock<'s, S>(
        &self,
        stream: S,
        name: Option<Self::Name>,
        lock_kind: LockKind,
    ) -> BoxStream<'s, S::Item>
    where
        S: Stream + Send + 's,
        <S as Stream>::Item: Send;
}
