//! I define podverse related concrete types for recipes.
//!

/// I define enumerated podverse related types.
pub mod enumerated {
    use std::sync::Arc;

    use manas_podverse::{
        pod::{
            impl_::BasicPod,
            service::impl_::{
                BasicPodService, BasicPodServiceFactory, StorageDescribingPodService,
                StorageDescribingPodServiceFactory,
            },
        },
        podset::{impl_::EnumeratedPodSet, service::impl_::BasicPodSetService},
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

    /// Type of enumerated podsets for recipes.
    pub type RcpEnumeratedPodSet<StSetup> = EnumeratedPodSet<RcpPod<StSetup>>;

    /// Type of enumerated podset services for recipes.
    pub type RcpEnumeratedPodSetService<StSetup> =
        BasicPodSetService<RcpEnumeratedPodSet<StSetup>, RcpPodServiceFactory<StSetup>>;

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

    impl<StSetup: RcpStorageSetup> CW<RcpEnumeratedPodSetService<StSetup>> {
        /// Get a new podset service serving enumerated pods.
        pub fn new_for_enumerated(
            pods: Vec<Arc<RcpPod<StSetup>>>,
            dev_mode: bool,
        ) -> RcpEnumeratedPodSetService<StSetup> {
            RcpEnumeratedPodSetService {
                pod_set: Arc::new(EnumeratedPodSet::new(pods)),
                pod_service_factory: Arc::new(CW::<RcpPodServiceFactory<StSetup>>::new(dev_mode)),
            }
        }
    }
}

/// I define assets overriden enumerated podverse related
/// types for the recipe.
pub mod assets_overriden_enumerated {
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

    use super::enumerated::{
        RcpEnumeratedPodSet, RcpEnumeratedPodSetService, RcpPod, RcpPodService,
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

    /// Type of enumerated podset service, with
    /// assets overriden pod services.
    pub type RcpEnumeratedAssetsOverridenPodSetService<RSetup> = BasicPodSetService<
        RcpEnumeratedPodSet<RSetup>,
        RcpAssetsOverridenPodServiceFactory<RSetup>,
    >;

    /// Type of assets overriden enumerated podset service.
    pub type RcpAssetsOverridenEnumeratedPodSetService<RSetup> =
        OverridenPodSetService<RcpEnumeratedPodSetService<RSetup>, RcpAssetsPodService>;

    impl<StSetup: RcpStorageSetup> CW<RcpAssetsOverridenEnumeratedPodSetService<StSetup>> {
        /// Get a new assets overriden pod set service serving
        /// enumerated pods
        pub fn new_for_enumerated(
            pods: Vec<Arc<RcpPod<StSetup>>>,
            overrider: RcpAssetsPodService,
            dev_mode: bool,
        ) -> RcpAssetsOverridenEnumeratedPodSetService<StSetup> {
            let inner =
                CW::<RcpEnumeratedPodSetService<StSetup>>::new_for_enumerated(pods, dev_mode);

            OverridenPodSetService::new(inner, overrider)
        }

        /// Get a new assets overriden pod set service serving
        /// single pod.
        pub fn new_for<Assets: RustEmbed + 'static>(
            pod: Arc<RcpPod<StSetup>>,
            assets_space: Arc<RcpStorageSpace>,
            assets_label: String,
            dev_mode: bool,
        ) -> RcpAssetsOverridenEnumeratedPodSetService<StSetup> {
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

            Self::new_for_enumerated(vec![pod], assets_pod_svc, dev_mode)
        }
    }
}
