//! I define a [`MethodService`] for handling `POST` method on solid resources.
//!

use self::{base::BasePostService, marshaller::default::DefaultBasePostResponseMarshaller};
use super::MethodService;

pub mod base;
pub mod marshaller;

/// Type alias for default `POST` service.
pub type DefaultPostService<Storage> =
    MethodService<BasePostService<Storage>, DefaultBasePostResponseMarshaller<Storage>>;
