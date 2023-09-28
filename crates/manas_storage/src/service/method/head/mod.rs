//! I define a [`MethodService`] for handling `HEAD` method on solid resources.
//!

use super::{
    get::{
        base::BaseGetService,
        marshaller::{
            default::DefaultBaseGetResponseMarshaller, head_only::HeadOnlyBaseGetResponseMarshaller,
        },
    },
    MethodService,
};

/// Type alias for default `Head` service.
pub type DefaultHeadService<Storage> = MethodService<
    BaseGetService<Storage>,
    HeadOnlyBaseGetResponseMarshaller<Storage, DefaultBaseGetResponseMarshaller<Storage>>,
>;
