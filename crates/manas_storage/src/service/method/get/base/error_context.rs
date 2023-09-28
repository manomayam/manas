//! I provide few types to represent error context in errors of base get method.
//!

use std::marker::PhantomData;

use manas_space::{
    resource::{slot::SolidResourceSlot, uri::SolidResourceUri},
    SolidStorageSpace,
};
use typed_record::TypedRecordKey;

/// A typed record key for target resource slot.
#[derive(Debug)]
pub struct KTargetResourceSlot<Space> {
    _phantom: PhantomData<Space>,
}

impl<P> Clone for KTargetResourceSlot<P> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Space: SolidStorageSpace> TypedRecordKey for KTargetResourceSlot<Space> {
    type Value = SolidResourceSlot<Space>;
}

/// A typed record key for existing mutex resource uri.
#[derive(Debug, Clone)]
pub struct KExistingMutexResourceUri;

impl TypedRecordKey for KExistingMutexResourceUri {
    type Value = SolidResourceUri;
}
