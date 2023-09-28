use std::num::NonZeroUsize;

use crate::{
    policy::aux::AuxPolicy,
    resource::slot_rel_type::aux_rel_type::known::impl_::default::DefaultKnownAuxRelType,
};

/// Default implementation of [`AuxPolicy`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultAuxPolicy;

impl AuxPolicy for DefaultAuxPolicy {
    type KnownAuxRelType = DefaultKnownAuxRelType;

    const PROV_PATH_MAX_AUX_LINKS: Option<NonZeroUsize> = None;
}
