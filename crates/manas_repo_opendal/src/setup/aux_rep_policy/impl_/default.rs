//! I define a default implementation of [`ODRAuxResourcePolicy`].

use std::{collections::HashSet, marker::PhantomData};

use manas_http::header::common::media_type::MediaType;
use manas_space::{
    policy::aux::impl_::DefaultAuxPolicy,
    resource::slot_rel_type::aux_rel_type::known::impl_::default::ALL_KNOWN_AUX_REL_TYPES,
    SolidStorageSpace, SpcKnownAuxRelType,
};
use once_cell::sync::Lazy;

use crate::setup::aux_rep_policy::ODRAuxResourcePolicy;

/// Default implementation of [`ODRAuxResourcePolicy`].
#[derive(Debug, Clone, Default)]
pub struct DefaultODRAuxResourcePolicy<Space: SolidStorageSpace> {
    _phantom: PhantomData<fn(Space)>,
}

static TURTLE_MEDIA_TYPE: Lazy<MediaType> =
    Lazy::new(|| "text/turtle".parse().expect("Must be valid"));

impl<Space> ODRAuxResourcePolicy for DefaultODRAuxResourcePolicy<Space>
where
    Space: SolidStorageSpace<AuxPolicy = DefaultAuxPolicy>,
{
    type StSpace = Space;

    const ALLOW_CUSTOM_AUX_REP_MEDIA_TYPE: bool = false;

    fn supported_aux_rel_types() -> &'static HashSet<SpcKnownAuxRelType<Self::StSpace>> {
        &ALL_KNOWN_AUX_REL_TYPES
    }

    #[inline]
    fn aux_rep_media_type(_: &SpcKnownAuxRelType<Space>) -> &'static MediaType {
        // For all aux resources, allow only turtle media type.
        Lazy::force(&TURTLE_MEDIA_TYPE)
    }
}
