//! I provide utilities to construct single pod recipes.
//!

// TODO Allow custom locker for distribution.

use std::{borrow::Cow, marker::PhantomData, sync::Arc};

use futures::future::{BoxFuture, TryFutureExt};
use http::uri::Scheme;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, MokaManager};
use manas_http::service::impl_::UriReconstructionParams;
use manas_repo::RepoExt;
use manas_repo_layers::dconneging::conneg_layer::impl_::binary_rdf_doc_converting::BinaryRdfDocContentNegotiationConfig;
use manas_repo_opendal::config::ODRConfig;
use manas_space::BoxError;
use manas_storage::service::impl_::{KPreferredReqTargetQueryParamMode, ReqTargetQueryParamMode};
use name_locker::impl_::InmemNameLocker;
use opendal::Builder as _;
use rdf_dynsyn::{
    parser::config::{
        jsonld::{HttpDocumentLoader, HttpDocumentLoaderOptions, JsonLdConfig},
        DynSynParserConfig,
    },
    serializer::config::DynSynSerializerConfig,
    DynSynFactorySet,
};
use sophia_turtle::serializer::turtle::TurtleConfig;
use tracing::error;
use typed_record::TypedRecord;

use self::{
    config::{RcpConfig, RcpStorageSpaceConfig},
    setup::SinglePodRecipeSetup,
};
use super::common::{resolve_authenticating_svc_maker, serve_recipe};
use crate::{
    dtbr::{adapt_dconneg_layer_config, DatabrowserContext, RcpDatabrowserAdaptedRdfSourceCNL},
    pep::{resolve_initial_root_acr_rep_factory, InitialRootAcrTemplateContext, RcpSimplePEP},
    podverse::static_::{RcpPod, RcpStaticPodSetService},
    recipe::Recipe,
    space::RcpStorageSpace,
    storage::{RcpStorage, RcpStorageSetup},
    CW,
};

pub mod config;
pub mod setup;

/// An implementation of [RcpStorageSetup`].
#[derive(Debug)]
pub struct SinglePodStorageSetup<RSetup> {
    _phantom: PhantomData<fn(RSetup)>,
}

impl<RSetup: SinglePodRecipeSetup> RcpStorageSetup for SinglePodStorageSetup<RSetup> {
    type Backend = RSetup::Backend;

    type ResourceLocker = InmemNameLocker<String>;

    type CNL = RcpDatabrowserAdaptedRdfSourceCNL<RSetup::Backend>;

    type PEP = RcpSimplePEP<RSetup::Backend, RSetup::PDP>;
}

/// An implementation of [`Recipe`] that serves single pod.
pub struct SinglePodRecipe<RSetup: SinglePodRecipeSetup> {
    _phantom: PhantomData<fn(RSetup)>,
}

impl<RSetup: SinglePodRecipeSetup> Default for SinglePodRecipe<RSetup> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

/// Type of storage for [`SinglePodRecipe`].
type SinglePodStorage<RSetup> = RcpStorage<SinglePodStorageSetup<RSetup>>;

impl<RSetup: SinglePodRecipeSetup> SinglePodRecipe<RSetup> {
    fn resolve_dynsyn_factory_set() -> DynSynFactorySet {
        let jsonld_doc_loader = HttpDocumentLoader::new(
            HttpDocumentLoaderOptions {
                max_redirections: 4,
                request_profile: Default::default(),
            },
            Some(Arc::new(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::default(),
                options: Default::default(),
            }))),
        );
        let loader_factory = Arc::new(move || jsonld_doc_loader.clone().into());
        let dynsyn_jsonld_config = JsonLdConfig {
            context_loader_factory: loader_factory,
            options: Default::default(),
        };

        let parsers_config =
            DynSynParserConfig::default().with_jsonld_config(dynsyn_jsonld_config.clone());

        let serializers_config = DynSynSerializerConfig::default()
            .with_jsonld_config(dynsyn_jsonld_config)
            // TODO should configure prefix map.
            .with_turtle_config(TurtleConfig::new().with_pretty(true));

        DynSynFactorySet::new_with_config(
            parsers_config.clone(),
            parsers_config,
            serializers_config.clone(),
            serializers_config,
        )
    }

    async fn resolve_initialized_pod(
        space_config: RcpStorageSpaceConfig,
        backend: RSetup::Backend,
        opt_databrowser_context: Option<DatabrowserContext>,
        pdp: Arc<RSetup::PDP>,
        initial_root_acr_template_str: &'static str,
    ) -> Result<RcpPod<SinglePodStorageSetup<RSetup>>, BoxError> {
        // Box::pin(async move {
        let st_descr_uri = format!("{}_/description.ttl", space_config.root_uri.as_str())
            .as_str()
            .parse()
            .unwrap();

        let databrowser_enabled = opt_databrowser_context.is_some();

        let st_space = CW::<RcpStorageSpace>::new_shared(
            space_config.root_uri.clone(),
            st_descr_uri,
            space_config.owner_id.clone(),
        );

        let dynsyn_factories = Arc::new(Self::resolve_dynsyn_factory_set());

        let mut storage = SinglePodStorage::<RSetup>::new_with_simple_pep(
            st_space,
            backend,
            ODRConfig {
                dynsyn_factories: dynsyn_factories.clone(),
                ..Default::default()
            },
            adapt_dconneg_layer_config(
                Arc::new(BinaryRdfDocContentNegotiationConfig {
                    dynsyn_factories: dynsyn_factories.as_ref().clone(),
                }),
                opt_databrowser_context,
            ),
            pdp,
            resolve_initial_root_acr_rep_factory(
                initial_root_acr_template_str,
                &InitialRootAcrTemplateContext {
                    storage_root_res_uri: space_config.root_uri,
                    owner_id: space_config.owner_id,
                },
            ),
            Default::default(),
        );

        // To let databrowser interpret redirect uris with
        // qparams on client side.
        if databrowser_enabled {
            storage
                .extensions
                .insert_rec_item::<KPreferredReqTargetQueryParamMode>(
                    ReqTargetQueryParamMode::Insignificant,
                );
        }

        // Initialize the repo.
        storage
            .repo
            .initialize()
            .inspect_err(|e| error!("Error in initializing the repo. Error:\n {}", e))
            .await?;

        Ok(RcpPod {
            storage: Arc::new(storage),
        })
        // })
    }
}

impl<RSetup: SinglePodRecipeSetup> Recipe for SinglePodRecipe<RSetup> {
    type Config = RcpConfig;

    fn cli_name(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "manas_server_single_{}_{}",
            RSetup::BACKEND_NAME,
            RSetup::PDP_NAME
        ))
    }

    fn description(&self) -> Cow<'static, str> {
        Cow::Owned(format!("Manas solid server that serves a single pod with {} backend and with {} access control system.", RSetup::BACKEND_NAME.to_uppercase(), RSetup::PDP_NAME.to_uppercase()))
    }

    fn serve(&self, config: Self::Config) -> BoxFuture<'static, Result<(), BoxError>> {
        Box::pin(async move {
            let space_config = config.storage.space.clone();
            let backend = RSetup::Backend::try_from(RSetup::BackendBuilder::from_map(
                config.storage.repo.backend,
            ))
            .map_err(|e| {
                error!("Error in resolving backend. Error: {}", e);
                e
            })?;

            let pod = Self::resolve_initialized_pod(
                space_config,
                backend,
                config
                    .storage
                    .repo
                    .databrowser_enabled
                    .then_some(DatabrowserContext::new_from_unpkg()),
                Default::default(),
                RSetup::INITIAL_ROOT_ACR_TEMPLATE,
            )
            .await?;

            let podset_svc = CW::<RcpStaticPodSetService<_>>::new_for_static(
                vec![Arc::new(pod)],
                config.dev_mode,
            );

            let uri_reconstruction_params = UriReconstructionParams {
                default_scheme: if config.server.tls.is_some() {
                    Scheme::HTTPS
                } else {
                    Scheme::HTTP
                },
                trusted_proxy_headers: config.server.trusted_proxy_headers.clone(),
            };

            let svc_maker = resolve_authenticating_svc_maker(podset_svc, uri_reconstruction_params);

            tracing::info!("Serving at {}", config.server.addr);
            tracing::info!(
                "Storage root uri: {}",
                config.storage.space.root_uri.as_str()
            );

            Ok(serve_recipe(config.server, svc_maker).await?)
        })
    }
}

#[cfg(all(feature = "backend-fs", feature = "pdp-wac"))]
/// Recipe that serves a single pod with FS backend, and WAC
/// access control system.
pub type SinglePodFsWacRecipe = SinglePodRecipe<setup::impl_::FsWacRecipeSetup>;

#[cfg(all(feature = "backend-fs", feature = "pdp-acp"))]
/// Recipe that serves a single pod with FS backend, and ACP
/// access control system.
pub type SinglePodFsAcpRecipe = SinglePodRecipe<setup::impl_::FsAcpRecipeSetup>;

#[cfg(all(feature = "backend-s3", feature = "pdp-wac"))]
/// Recipe that serves a single pod with S3 backend, and WAC
/// access control system.
pub type SinglePodS3WacRecipe = SinglePodRecipe<setup::impl_::S3WacRecipeSetup>;

#[cfg(all(feature = "backend-s3", feature = "pdp-acp"))]
/// Recipe that serves a single pod with S3 backend, and ACP
/// access control system.
pub type SinglePodS3AcpRecipe = SinglePodRecipe<setup::impl_::S3AcpRecipeSetup>;

#[cfg(all(feature = "backend-gcs", feature = "pdp-wac"))]
/// Recipe that serves a single pod with GCS backend, and WAC
/// access control system.
pub type SinglePodGcsWacRecipe = SinglePodRecipe<setup::impl_::GcsWacRecipeSetup>;

#[cfg(all(feature = "backend-gcs", feature = "pdp-acp"))]
/// Recipe that serves a single pod with GCS backend, and ACP
/// access control system.
pub type SinglePodGcsAcpRecipe = SinglePodRecipe<setup::impl_::GcsAcpRecipeSetup>;
