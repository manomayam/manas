//! I define a [`MethodService`] for handling `GET` method on solid resources.
//!

use self::{base::BaseGetService, marshaller::default::DefaultBaseGetResponseMarshaller};
use super::MethodService;

pub mod base;

pub mod marshaller;

/// Type alias for default `GET` service.
pub type DefaultGetService<Storage> =
    MethodService<BaseGetService<Storage>, DefaultBaseGetResponseMarshaller<Storage>>;
