//! I define a [`MethodService`] for handling `PUT` / `PATCH` method on solid resources.
//!

use self::{
    base::BasePutOrPatchService, marshaller::default::DefaultBasePutOrPatchResponseMarshaller,
};
use super::MethodService;

pub mod base;
pub mod marshaller;

/// Type alias for default `PutOrPatch` service.
pub type DefaultPutOrPatchService<Storage> =
    MethodService<BasePutOrPatchService<Storage>, DefaultBasePutOrPatchResponseMarshaller<Storage>>;
