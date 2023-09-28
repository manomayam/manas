//! I define concrete types for the storages for recipes.
//!

use std::{collections::HashSet, fmt::Debug, marker::PhantomData, sync::Arc};

use frunk_core::hlist;
use manas_access_control::{
    layered_repo::context::AccessControlledRepoContext,
    model::{pdp::PolicyDecisionPoint, pep::PolicyEnforcementPoint},
};
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_repo::{context::RepoContextual, Repo};
use manas_repo_layers::{
    dconneging::{
        conneg_layer::DerivedContentNegotiationLayer, context::DerivedContentNegotiatingRepoContext,
    },
    patching::{
        context::PatchingRepoContext,
        patcher::impl_::{
            binary_rdf_doc_patcher::BinaryRdfDocPatcherResolutionConfig,
            solid_insert_delete_patcher::SolidInsertDeletePatcherResolutionConfig,
        },
    },
    validating::{
        context::ValidatingRepoContext,
        update_validator::impl_::{
            common::RdfSourceRepUpdateValidatorConfig, multi::MultiRepUpdateValidatorConfig,
        },
    },
};
use manas_repo_opendal::{
    config::ODRConfig, context::ODRContext, object_store::backend::ODRObjectStoreBackend,
    service::resource_operator::reader::ODRResourceReader,
};
use manas_storage::{
    policy::method::impl_::RdfPatchingMethodPolicy,
    service::impl_::{DefaultStorageService, DefaultStorageServiceFactory},
    SolidStorage,
};
use name_locker::NameLocker;
use rdf_utils::model::triple::ArcTriple;

use crate::{
    pep::{InitialRootAcrRepFactory, RcpPRP, RcpSimplePEP, SimpleAccessRcpStorageSetup},
    repo::{RcpBaseRepo, RcpBaseRepoSetup, RcpCNLConfig, RcpRepo},
    space::RcpStorageSpace,
};

/// A trait for concrete setup of the recipe storage.
pub trait RcpStorageSetup: Debug + Send + 'static + Sized {
    /// Type of object store backend.
    type Backend: ODRObjectStoreBackend;

    /// Type of resource locker.
    type ResourceLocker: NameLocker<Name = String> + Unpin;

    /// Type of he content negotiation layer.
    type CNL: DerivedContentNegotiationLayer<
        RcpBaseRepo<Self::Backend>,
        BinaryRepresentation,
        ODRResourceReader<RcpBaseRepoSetup<Self::Backend>>,
    >;

    /// Type of policy enforcement point.
    type PEP: PolicyEnforcementPoint<StSpace = RcpStorageSpace>;
}

/// A generic implementation of [`RcpStorageSetup`].
pub struct GenericRcpStorageSetup<Backend, CNL, RL, PEP> {
    _phantom: PhantomData<fn(Backend, CNL, RL, PEP)>,
}

impl<Backend, CNL, RL, PEP> Debug for GenericRcpStorageSetup<Backend, CNL, RL, PEP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericRcpStorageSetup").finish()
    }
}

impl<Backend, CNL, RL, PEP> RcpStorageSetup for GenericRcpStorageSetup<Backend, CNL, RL, PEP>
where
    Backend: ODRObjectStoreBackend,
    CNL: DerivedContentNegotiationLayer<
        RcpBaseRepo<Backend>,
        BinaryRepresentation,
        ODRResourceReader<RcpBaseRepoSetup<Backend>>,
    >,
    RL: NameLocker<Name = String> + Unpin,
    PEP: PolicyEnforcementPoint<StSpace = RcpStorageSpace>,
{
    type Backend = Backend;

    type CNL = CNL;

    type ResourceLocker = RL;

    type PEP = PEP;
}

/// An implementation of the [`SolidStorage`] for the recipe.
pub struct RcpStorage<StSetup: RcpStorageSetup> {
    /// Method policy of the storage.
    pub method_policy: RdfPatchingMethodPolicy,

    /// Repo for the storage.
    pub repo: RcpRepo<StSetup::Backend, StSetup::CNL, StSetup::PEP>,

    /// Resource locker.
    pub resource_locker: StSetup::ResourceLocker,

    /// Any extensions.
    pub extensions: http::Extensions,
}

impl<StSetup: RcpStorageSetup> SolidStorage for RcpStorage<StSetup> {
    type StSpace = RcpStorageSpace;

    // With rdf patching method policy.
    type MethodPolicy = RdfPatchingMethodPolicy;

    type Repo = RcpRepo<StSetup::Backend, StSetup::CNL, StSetup::PEP>;

    type ResourceLocker = StSetup::ResourceLocker;

    #[inline]
    fn method_policy(&self) -> &Self::MethodPolicy {
        &self.method_policy
    }

    #[inline]
    fn repo(&self) -> &Self::Repo {
        &self.repo
    }

    #[inline]
    fn resource_locker(&self) -> &Self::ResourceLocker {
        &self.resource_locker
    }

    #[inline]
    fn extensions(&self) -> &http::Extensions {
        &self.extensions
    }
}

impl<StSetup: RcpStorageSetup> RcpStorage<StSetup> {
    pub(crate) fn _new(
        odr_context: Arc<ODRContext<RcpBaseRepoSetup<StSetup::Backend>>>,
        conneg_layer_config: Arc<RcpCNLConfig<StSetup::CNL, StSetup::Backend>>,
        pep: StSetup::PEP,
        initial_root_acr_rep_factory: InitialRootAcrRepFactory,
        resource_locker: StSetup::ResourceLocker,
    ) -> Self {
        let dynsyn_factories = odr_context.as_ref().config.dynsyn_factories.clone();

        let patcher_resolution_config = Arc::new(BinaryRdfDocPatcherResolutionConfig {
            dynsyn_factories: dynsyn_factories.clone(),
            inner: Arc::new(SolidInsertDeletePatcherResolutionConfig {
                dynsyn_parser_factories: dynsyn_factories.as_ref().parser.clone(),
                max_patch_doc_payload_size: Some(4 * 1024 * 1024),
            }),
        });

        let rdf_source_rep_validator_config = Arc::new(RdfSourceRepUpdateValidatorConfig {
            dynsyn_parser_factories: dynsyn_factories.as_ref().parser.clone(),
            max_user_supplied_rep_size: Some(8 * 1024 * 1024),
        });

        let rep_update_validator_config = Arc::new(MultiRepUpdateValidatorConfig::new(hlist![
            rdf_source_rep_validator_config.clone(),
            rdf_source_rep_validator_config
        ]));

        let repo_context = Arc::new(AccessControlledRepoContext {
            pep,
            inner: Arc::new(PatchingRepoContext {
                inner: Arc::new(ValidatingRepoContext {
                    inner: Arc::new(DerivedContentNegotiatingRepoContext {
                        inner: odr_context,
                        dconneg_layer_config: conneg_layer_config,
                    }),
                    rep_update_validator_config,
                }),
                patcher_resolution_config,
            }),
            initial_root_acr_rep_factory,
        });

        Self {
            method_policy: Default::default(),
            repo: RcpRepo::new(repo_context),
            resource_locker,
            extensions: Default::default(),
        }
    }

    /// Create a new [`RcpStorage`] with given params.
    pub fn new(
        storage_space: Arc<RcpStorageSpace>,
        backend: StSetup::Backend,
        odr_config: ODRConfig,
        conneg_layer_config: Arc<RcpCNLConfig<StSetup::CNL, StSetup::Backend>>,
        pep: StSetup::PEP,
        initial_root_acr_rep_factory: InitialRootAcrRepFactory,
        resource_locker: StSetup::ResourceLocker,
    ) -> Self {
        let odr_context = Arc::new(ODRContext::new(storage_space, backend, odr_config));

        Self::_new(
            odr_context,
            conneg_layer_config,
            pep,
            initial_root_acr_rep_factory,
            resource_locker,
        )
    }

    /// Create a new [`RcpStorage`] with [``RcpSimplePEP`] as pep..
    pub fn new_with_simple_pep<
        Backend: ODRObjectStoreBackend,
        PDP: PolicyDecisionPoint<StSpace = RcpStorageSpace, Graph = HashSet<ArcTriple>>,
    >(
        storage_space: Arc<RcpStorageSpace>,
        backend: StSetup::Backend,
        odr_config: ODRConfig,
        conneg_layer_config: Arc<RcpCNLConfig<StSetup::CNL, StSetup::Backend>>,
        pdp: Arc<PDP>,
        initial_root_acr_rep_factory: InitialRootAcrRepFactory,
        resource_locker: StSetup::ResourceLocker,
    ) -> Self
    where
        StSetup: SimpleAccessRcpStorageSetup<Backend_ = Backend, PDP = PDP>,
    {
        let odr_context = Arc::new(ODRContext::new(storage_space.clone(), backend, odr_config));

        let pep = RcpSimplePEP {
            storage_space,
            pdp,
            prp: Arc::new(RcpPRP::<Backend>::new_with_context(odr_context.clone())),
        };

        Self::_new(
            odr_context,
            conneg_layer_config,
            pep,
            initial_root_acr_rep_factory,
            resource_locker,
        )
    }
}

/// Type of storage services for the recipes.
pub type RcpStorageService<StSetup> = DefaultStorageService<RcpStorage<StSetup>>;

/// Type of storage service factories for the recipes.
pub type RcpStorageServiceFactory<StSetup> = DefaultStorageServiceFactory<RcpStorage<StSetup>>;
