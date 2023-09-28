//! I define the default implementation of [`ODRObjectSpaceSetup``].
//!

use manas_semslot::scheme::impl_::hierarchical::{
    aux::impl_::default::DefaultAuxLinkEncodingScheme, HierarchicalSemanticSlotEncodingScheme,
};
use manas_space::impl_::DefaultSolidStorageSpace;

use crate::object_store::object_space::{
    assoc::mapping_scheme::impl_::default::DefaultAssocMappingScheme, ODRObjectSpace,
    ODRObjectSpaceSetup,
};

/// An implementation of [`ODRObjectSpaceSetup`] for the
/// recipe.
#[derive(Debug, Clone)]
pub struct DefaultODRObjectSpaceSetup {}

impl ODRObjectSpaceSetup for DefaultODRObjectSpaceSetup {
    type AssocStSpace = DefaultSolidStorageSpace;

    type AssocStSemSlotES = HierarchicalSemanticSlotEncodingScheme<
        DefaultSolidStorageSpace,
        DefaultAuxLinkEncodingScheme,
    >;

    type AssocMappingScheme =
        DefaultAssocMappingScheme<DefaultSolidStorageSpace, DefaultAuxLinkEncodingScheme>;
}

/// Type of default odr object space.
pub type DefaultODRObjectSpace = ODRObjectSpace<DefaultODRObjectSpaceSetup>;
