//! I define podverse related concrete types for recipes.
//!

/// I define static podverse related types.
pub mod static_ {
    use std::sync::Arc;

    use manas_podverse::{
        pod::{
            impl_::BasicPod,
            service::impl_::{
                BasicPodService, BasicPodServiceFactory, StorageDescribingPodService,
                StorageDescribingPodServiceFactory,
            },
        },
        podset::{impl_::static_::StaticPodSet, service::impl_::BasicPodSetService},
    };
    use manas_storage::service::impl_::DefaultStorageServiceFactory;

    use crate::{
        storage::{RcpStorage, RcpStorageService, RcpStorageServiceFactory, RcpStorageSetup},
        CW,
    };

    /// Type of pods for recipes.
    pub type RcpPod<StSetup> = BasicPod<RcpStorage<StSetup>>;

    /// Type of pod services for recipes.
    pub type RcpPodService<StSetup> =
        StorageDescribingPodService<BasicPodService<RcpPod<StSetup>, RcpStorageService<StSetup>>>;

    /// Type of pod service factories for recipes.
    pub type RcpPodServiceFactory<StSetup> = StorageDescribingPodServiceFactory<
        BasicPodServiceFactory<RcpPod<StSetup>, RcpStorageServiceFactory<StSetup>>,
    >;

    /// Type of static podsets for recipes.
    pub type RcpStaticPodSet<StSetup> = StaticPodSet<RcpPod<StSetup>>;

    /// Type of static podset services for recipes.
    pub type RcpStaticPodSetService<StSetup> =
        BasicPodSetService<RcpStaticPodSet<StSetup>, RcpPodServiceFactory<StSetup>>;

    impl<StSetup: RcpStorageSetup> CW<RcpPodServiceFactory<StSetup>> {
        /// Get a new pod service factory.
        #[allow(clippy::new_ret_no_self)]
        pub fn new(dev_mode: bool) -> RcpPodServiceFactory<StSetup> {
            StorageDescribingPodServiceFactory {
                inner_factory: Arc::new(BasicPodServiceFactory::new(Arc::new(
                    DefaultStorageServiceFactory::new(dev_mode),
                ))),
            }
        }
    }

    impl<StSetup: RcpStorageSetup> CW<RcpStaticPodSetService<StSetup>> {
        /// Get a new podset service serving static pods.
        pub fn new_for_static(
            pods: Vec<Arc<RcpPod<StSetup>>>,
            dev_mode: bool,
        ) -> RcpStaticPodSetService<StSetup> {
            RcpStaticPodSetService {
                pod_set: Arc::new(StaticPodSet::new(pods)),
                pod_service_factory: Arc::new(CW::<RcpPodServiceFactory<StSetup>>::new(dev_mode)),
            }
        }
    }
}

/// I define assets overriden static podverse related
/// types for the recipe.
pub mod assets_overriden_static {
    use std::sync::Arc;

    use manas_podverse::{
        pod::{
            impl_::BasicPod,
            service::{
                impl_::{OverridenPodService, OverridenPodServiceFactory},
                PodServiceFactory,
            },
        },
        podset::service::impl_::{BasicPodSetService, OverridenPodSetService},
    };
    use manas_repo_opendal::object_store::backend::impl_::embedded::{
        service::Embedded, EmbeddedBackend,
    };
    use name_locker::impl_::VoidNameLocker;
    use rust_embed::RustEmbed;

    use super::static_::{
        RcpStaticPodSet, RcpStaticPodSetService, RcpPod, RcpPodService,
        RcpPodServiceFactory,
    };
    use crate::{
        pep::RcpTrivialPEP,
        repo::RcpRdfSourceCNL,
        space::RcpStorageSpace,
        storage::{GenericRcpStorageSetup, RcpStorage, RcpStorageSetup},
        CW,
    };

    type MBackend = EmbeddedBackend;

    /// Type of embedded storage setup for the recipe.
    pub type RcpEmbeddedStorageSetup = GenericRcpStorageSetup<
        MBackend,
        RcpRdfSourceCNL<MBackend>,
        VoidNameLocker<String>,
        RcpTrivialPEP,
    >;

    /// Type of embedded assets pods.
    pub type RcpAssetsPod = BasicPod<RcpStorage<RcpEmbeddedStorageSetup>>;

    /// Type of embedded assets pod services.
    pub type RcpAssetsPodService = RcpPodService<RcpEmbeddedStorageSetup>;

    /// Type of pod services with an inner pod service
    /// route-overriden by an assets pod service.
    pub type RcpAssetsOverridenPodService<RSetup> =
        OverridenPodService<RcpPodService<RSetup>, RcpAssetsPodService>;

    /// Type of assets-overriden-pod-service factory.
    pub type RcpAssetsOverridenPodServiceFactory<RSetup> = OverridenPodServiceFactory<
        RcpPodServiceFactory<RSetup>,
        RcpPodService<RcpEmbeddedStorageSetup>,
    >;

    /// Type of static podset service, with
    /// assets overriden pod services.
    pub type RcpStaticAssetsOverridenPodSetService<RSetup> = BasicPodSetService<
        RcpStaticPodSet<RSetup>,
        RcpAssetsOverridenPodServiceFactory<RSetup>,
    >;

    /// Type of assets overriden static podset service.
    pub type RcpAssetsOverridenStaticPodSetService<RSetup> =
        OverridenPodSetService<RcpStaticPodSetService<RSetup>, RcpAssetsPodService>;

    impl<StSetup: RcpStorageSetup> CW<RcpAssetsOverridenStaticPodSetService<StSetup>> {
        /// Get a new assets overriden pod set service serving
        /// static pods
        pub fn new_for_static(
            pods: Vec<Arc<RcpPod<StSetup>>>,
            overrider: RcpAssetsPodService,
            dev_mode: bool,
        ) -> RcpAssetsOverridenStaticPodSetService<StSetup> {
            let inner =
                CW::<RcpStaticPodSetService<StSetup>>::new_for_static(pods, dev_mode);

            OverridenPodSetService::new(inner, overrider)
        }

        /// Get a new assets overriden pod set service serving
        /// single pod.
        pub fn new_for<Assets: RustEmbed + 'static>(
            pod: Arc<RcpPod<StSetup>>,
            assets_space: Arc<RcpStorageSpace>,
            assets_label: String,
            dev_mode: bool,
        ) -> RcpAssetsOverridenStaticPodSetService<StSetup> {
            let assets_backend =
                EmbeddedBackend::try_from(Embedded::<Assets>::default().with_name(assets_label))
                    .expect("Must be valid");

            let assets_storage = RcpStorage::<RcpEmbeddedStorageSetup>::new(
                assets_space,
                assets_backend,
                Default::default(),
                Default::default(),
                Default::default(),
                Arc::new(|_| None),
                Default::default(),
            );

            let assets_pod = BasicPod {
                storage: Arc::new(assets_storage),
            };

            let assets_pod_svc = CW::<RcpPodServiceFactory<RcpEmbeddedStorageSetup>>::new(dev_mode)
                .new_service(Arc::new(assets_pod));

            Self::new_for_static(vec![pod], assets_pod_svc, dev_mode)
        }
    }
}
