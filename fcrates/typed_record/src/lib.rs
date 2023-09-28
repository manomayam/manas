//! This crate defines traits for handling typed-key-value-records easily.
//!

#![warn(missing_docs)]
#![deny(unused_qualifications)]

use std::marker::PhantomData;

/// A trait for keys in [`TypedRecord`].
pub trait TypedRecordKey: Send + Sync + 'static + Clone {
    /// Associated value type.
    type Value: Send + Sync + 'static + Clone;
}
/// A typed record is a record of heterogenous items, where key-type
/// points to corresponding value type as it's associated type.
pub trait TypedRecord {
    /// Get reference to value of record with given type key..
    fn get_rv<RK: TypedRecordKey>(&self) -> Option<&RK::Value>;

    /// Get mutable reference to value of record with given type key.
    fn get_rv_mut<RK: TypedRecordKey>(&mut self) -> Option<&mut RK::Value>;

    /// Insert record item with given typed key, and a value.
    fn insert_rec_item<RK: TypedRecordKey>(&mut self, v: RK::Value) -> Option<RK::Value>;

    /// Delete record item with given typed key.
    /// Returns any existing value.
    fn remove_rec_item<RK: TypedRecordKey>(&mut self) -> Option<RK::Value>;

    /// Get back record with given kv pair inserted.
    #[inline]
    fn with_rec_item<RK: TypedRecordKey>(mut self, v: RK::Value) -> Self
    where
        Self: Sized,
    {
        self.insert_rec_item::<RK>(v);
        self
    }

    /// Get back record with given optional kv pair inserted.
    #[inline]
    fn with_rec_item_opt<K: TypedRecordKey>(mut self, v: Option<K::Value>) -> Self
    where
        Self: Sized,
    {
        if let Some(v) = v {
            self.insert_rec_item::<K>(v);
        }
        self
    }
}

/// A PhantomKey is zero sized type which is generic over a [`TypedRecordKey`].
#[derive(Debug, Clone)]
pub struct PhantomKey<K: TypedRecordKey> {
    _phantom: PhantomData<fn() -> K>,
}

impl<K: TypedRecordKey> Default for PhantomKey<K> {
    #[inline]
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// An entry for using in extension maps that implement [`TypedRecord`].
#[derive(Debug, Clone)]
pub struct TypedRecordEntry<K: TypedRecordKey> {
    /// Phantom key
    pub k: PhantomKey<K>,

    /// Value.
    pub v: K::Value,
}

impl<K: TypedRecordKey> TypedRecordEntry<K> {
    /// Create a new [`TypedRecordEntry`] with given entry-value.
    #[inline]
    pub fn new(v: K::Value) -> Self {
        Self {
            k: Default::default(),
            v,
        }
    }
}

#[cfg(feature = "ext-http")]
impl TypedRecord for http::Extensions {
    #[inline]
    fn get_rv<K: TypedRecordKey>(&self) -> Option<&K::Value> {
        self.get::<TypedRecordEntry<K>>().map(|e| &e.v)
    }

    #[inline]
    fn get_rv_mut<K: TypedRecordKey>(&mut self) -> Option<&mut K::Value> {
        self.get_mut::<TypedRecordEntry<K>>().map(|e| &mut e.v)
    }

    #[inline]
    fn insert_rec_item<K: TypedRecordKey>(&mut self, v: K::Value) -> Option<K::Value> {
        self.insert(TypedRecordEntry::<K>::new(v)).map(|e| e.v)
    }

    #[inline]
    fn remove_rec_item<K: TypedRecordKey>(&mut self) -> Option<K::Value> {
        self.remove::<TypedRecordEntry<K>>().map(|e| e.v)
    }
}

#[cfg(feature = "ext-anymap")]
/// A type alias for Clonable concurrent safe  valued any map.
pub type ClonableTypedRecord =
    anymap2::Map<dyn anymap2::any::CloneAnySendSync + Sync + Send + 'static>;

#[cfg(feature = "ext-anymap")]
impl TypedRecord for ClonableTypedRecord {
    #[inline]
    fn get_rv<K: TypedRecordKey>(&self) -> Option<&K::Value> {
        self.get::<TypedRecordEntry<K>>().map(|e| &e.v)
    }

    #[inline]
    fn get_rv_mut<K: TypedRecordKey>(&mut self) -> Option<&mut K::Value> {
        self.get_mut::<TypedRecordEntry<K>>().map(|e| &mut e.v)
    }

    #[inline]
    fn insert_rec_item<K: TypedRecordKey>(&mut self, v: K::Value) -> Option<K::Value> {
        self.insert(TypedRecordEntry::<K>::new(v)).map(|e| e.v)
    }

    #[inline]
    fn remove_rec_item<K: TypedRecordKey>(&mut self) -> Option<K::Value> {
        self.remove::<TypedRecordEntry<K>>().map(|e| e.v)
    }
}
