//! I define concrete types for the repos for recipes.
//!

use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

use frunk_core::HList;
use manas_access_control::layered_repo::AccessControlledRepo;
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_repo_layers::{
    dconneging::{
        conneg_layer::{
            impl_::binary_rdf_doc_converting::BinaryRdfDocContentNegotiationLayer,
            DerivedContentNegotiationLayer,
        },
        DerivedContentNegotiatingRepo,
    },
    patching::{
        patcher::impl_::{
            binary_rdf_doc_patcher::BinaryRdfDocPatcher,
            solid_insert_delete_patcher::SolidInsertDeletePatcher,
        },
        PatchingRepo,
    },
    validating::{
        update_validator::impl_::{
            aux_protecting::AuxProtectingRepUpdateValidator,
            container_protecting::ContainerProtectingRepUpdateValidator,
            multi::MultiRepUpdateValidator,
        },
        ValidatingRepo,
    },
};
use manas_repo_opendal::{
    object_store::{
        backend::ODRObjectStoreBackend,
        object_space::{
            assoc::mapping_scheme::impl_::default::DefaultAssocMappingScheme, ODRObjectSpaceSetup,
        },
        setup::impl_::BasicODRObjectStoreSetup,
    },
    service::resource_operator::reader::ODRResourceReader,
    setup::{aux_rep_policy::impl_::default::DefaultODRAuxResourcePolicy, ODRSetup},
    OpendalRepo,
};
use manas_semslot::scheme::impl_::hierarchical::{
    aux::impl_::default::DefaultAuxLinkEncodingScheme, HierarchicalSemanticSlotEncodingScheme,
};
use rdf_utils::model::quad::ArcQuad;

use crate::space::RcpStorageSpace;

/// An implementation of [`ODRObjectSpaceSetup`] for the
/// recipes.
#[derive(Debug, Clone)]
pub struct RcpObjectSpaceSetup {}

impl ODRObjectSpaceSetup for RcpObjectSpaceSetup {
    type AssocStSpace = RcpStorageSpace;

    // Recipe uses hierarchical encoded semslots.
    type AssocStSemSlotES =
        HierarchicalSemanticSlotEncodingScheme<RcpStorageSpace, DefaultAuxLinkEncodingScheme>;

    type AssocMappingScheme =
        DefaultAssocMappingScheme<RcpStorageSpace, DefaultAuxLinkEncodingScheme>;
}

/// Type of odr object store setup for the recipes.
pub type RcpObjectStoreSetup<Backend> = BasicODRObjectStoreSetup<RcpObjectSpaceSetup, Backend>;

/// An implementation of [`ODRSetup`] for the recipe's base
/// repo.
pub struct RcpBaseRepoSetup<Backend> {
    _phantom: PhantomData<fn(Backend)>,
}

impl<Backend> Debug for RcpBaseRepoSetup<Backend> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RcpBaseRepoSetup").finish()
    }
}

impl<Backend> Clone for RcpBaseRepoSetup<Backend> {
    fn clone(&self) -> Self {
        Self {
            _phantom: self._phantom,
        }
    }
}

impl<Backend: ODRObjectStoreBackend> ODRSetup for RcpBaseRepoSetup<Backend> {
    type StSpace = RcpStorageSpace;

    type ObjectStoreSetup = RcpObjectStoreSetup<Backend>;

    type AuxResourcePolicy = DefaultODRAuxResourcePolicy<RcpStorageSpace>;
}

/// Type of base opendal repo for the recipe.
pub type RcpBaseRepo<Backend> = OpendalRepo<RcpBaseRepoSetup<Backend>>;

/// Type of conneg layered repo for the recipe.
pub type RcpConnegingRepo<Backend, CNL> = DerivedContentNegotiatingRepo<RcpBaseRepo<Backend>, CNL>;

/// Type of rep validator for the recipe.
pub type RcpRepValidator<Backend, CNL> = MultiRepUpdateValidator<
    RcpConnegingRepo<Backend, CNL>,
    HList!(
        // Validation layer that ensures validity of container reps.
        ContainerProtectingRepUpdateValidator<RcpConnegingRepo<Backend, CNL>, BinaryRepresentation>,
        // Validation layer that ensures validity of aux resource reps.
        AuxProtectingRepUpdateValidator<RcpConnegingRepo<Backend, CNL>, BinaryRepresentation>,
    ),
>;

/// Type of representation patcher for the recipe.
pub type RcpRepPatcher = BinaryRdfDocPatcher<
    RcpStorageSpace,
    // N3 solid-insert-delete patcher.
    SolidInsertDeletePatcher<RcpStorageSpace, HashSet<ArcQuad>>,
    HashSet<ArcQuad>,
>;

/// Type of the repo for the recipe.
/// Recipe uses access-control, rep-patching, rep-validating, and
/// conneg layered opendal repo as it's repo.
pub type RcpRepo<Backend, CNL, PEP> = AccessControlledRepo<
    PatchingRepo<
        ValidatingRepo<RcpConnegingRepo<Backend, CNL>, RcpRepValidator<Backend, CNL>>,
        RcpRepPatcher,
    >,
    PEP,
>;

/// Type of rdf source conneg layer for recipe.
pub type RcpRdfSourceCNL<Backend> = BinaryRdfDocContentNegotiationLayer<
    RcpBaseRepo<Backend>,
    ODRResourceReader<RcpBaseRepoSetup<Backend>>,
>;

/// Type of recipe's content negotiation config.
pub type RcpCNLConfig<CNL, Backend> = <CNL as DerivedContentNegotiationLayer<
    RcpBaseRepo<Backend>,
    BinaryRepresentation,
    ODRResourceReader<RcpBaseRepoSetup<Backend>>,
>>::Config;
