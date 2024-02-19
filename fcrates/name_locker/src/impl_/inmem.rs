//! I define an inmemory implementation of [`NameLocker`].
//!

use std::{future::Future, hash::Hash, sync::Arc};

use async_stream::stream;
use dashmap::DashMap;
use futures::{future::BoxFuture, stream::BoxStream, Stream};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{LockKind, NameLocker};

/// An enum to hold lock guard.
enum LockGuard<'g> {
    Read(RwLockReadGuard<'g, ()>),
    Write(RwLockWriteGuard<'g, ()>),
}

/// An implementation of [`NameLocker`], that uses inmemory lock table.
///
/// As this uses inmemory lock table, it cannot lock a name across different processes.
///
#[derive(Debug)]
pub struct InmemNameLocker<Name>
where
    Name: Ord + Hash + Clone + Send + Sync + 'static,
{
    lock_table: Arc<DashMap<Name, Arc<RwLock<()>>>>,
}

impl<Name> Default for InmemNameLocker<Name>
where
    Name: Ord + Hash + Clone + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<Name> InmemNameLocker<Name>
where
    Name: Ord + Hash + Clone + Send + Sync + 'static,
{
    /// Create a new in memory name locker.
    #[inline]
    pub fn new() -> Self {
        Self {
            lock_table: Arc::new(DashMap::new()),
        }
    }

    /// Get or insert a lock over given name atomically.
    #[inline]
    fn get_or_insert_lock(&self, name: Name) -> Arc<RwLock<()>> {
        // Atomically get or insert.
        self.lock_table
            .entry(name) // TODO should prevent allocation here.
            .or_insert(Arc::new(RwLock::new(())))
            .clone()
    }

    /// Remove lock over given name from given lock table,
    /// if there is no contention over the name.
    fn _remove_lock_if_no_contention(locks: Arc<DashMap<Name, Arc<RwLock<()>>>>, name: &Name) {
        let mut guard = None;
        locks.remove_if_mut(name, |_, v| {
            // Ensure no other strong refs to arced rwlock are already taken
            if Arc::strong_count(v) == 1 {
                // Ensure no other task is accessing resource.
                // May be redundant(?) after above arc check. as every access guard requires an Arc in `InmemResourceLocker` implementation.
                if let Ok(g) = v.clone().try_write_owned() {
                    // Store guard for duration of this operation.
                    guard = Some(g);
                    // Remove, as there is no contention for given name
                    return true;
                }
            }
            false
        });
    }

    /*
    /// Removes lock entry, if there is no contention.
    #[inline]
    fn remove_lock_if_no_contention(&self, resource_id: &ResourceId) {
        Self::_remove_lock_if_no_contention(self.locks.clone(), resource_id);
    }
    */
}

impl<Name> NameLocker for InmemNameLocker<Name>
where
    Name: Ord + Hash + Clone + Send + Sync + 'static,
{
    type Name = Name;

    fn poll_with_lock<Output, Task>(
        &self,
        task: Task,
        name: Option<Self::Name>,
        lock_kind: LockKind,
    ) -> BoxFuture<'static, Output>
    where
        Task: Future<Output = Output> + Send + 'static,
    {
        if let Some(name) = name {
            let lock_table = self.lock_table.clone();
            // Get lock over name.
            let name_lock = self.get_or_insert_lock(name.clone());

            Box::pin(async move {
                // Acquire specified lock over name.
                // This guard lasts across an await point.
                let name_guard = match lock_kind {
                    LockKind::Shared => LockGuard::Read(name_lock.read().await),
                    LockKind::Exclusive => LockGuard::Write(name_lock.write().await),
                };

                // Await task
                let output = task.await;

                // Drop guard and arced lock explicitly..
                drop(name_guard);
                drop(name_lock);

                // Cleanup lock if no contention.
                Self::_remove_lock_if_no_contention(lock_table, &name);
                output
            })
        } else {
            // If name is `None`, directly await task.
            Box::pin(task)
        }
    }

    fn poll_read_with_lock<'s, S>(
        &self,
        in_stream: S,
        name: Option<Self::Name>,
        lock_kind: LockKind,
    ) -> BoxStream<'s, S::Item>
    where
        S: Stream + Send + 's,
        <S as Stream>::Item: Send,
    {
        if let Some(name) = name {
            // Get lock over name.
            let name_lock = self.get_or_insert_lock(name.clone());
            let lock_table = self.lock_table.clone();

            Box::pin(stream! {
                // Acquire specified lock over name.
                // This guard lasts across an await point.
                let name_guard = match lock_kind {
                    LockKind::Shared => LockGuard::Read(name_lock.read().await),
                    LockKind::Exclusive => LockGuard::Write(name_lock.write().await),
                };

                // Yield items.
                for await item in in_stream {
                    yield item;
                }

                // Drop guard and arced lock explicitly..
                drop(name_guard);
                drop(name_lock);

                // Cleanup lock if no contention.
                Self::_remove_lock_if_no_contention(lock_table, &name);
            })
        } else {
            // If name is `None`, directly return stream.
            Box::pin(in_stream)
        }
    }
}
