//! I define a [`MethodService`] for handling `DELETE` method on solid resources.
//!

use self::{base::BaseDeleteService, marshaller::default::DefaultBaseDeleteResponseMarshaller};
use super::MethodService;

pub mod base;
pub mod marshaller;

/// Type alias for default `DELETE` service.
pub type DefaultDeleteService<Storage> =
    MethodService<BaseDeleteService<Storage>, DefaultBaseDeleteResponseMarshaller<Storage>>;
