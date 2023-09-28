//! I define default implementation of [`SolidStorageService`](super::SolidStorageService).
//!

use std::{collections::HashMap, convert::Infallible, marker::PhantomData, sync::Arc, task::Poll};

use dyn_problem::{type_::INFALLIBLE, Problem};
use futures::future::BoxFuture;
use http::{Method, Request, Response};
use manas_http::{
    service::{
        impl_::{NormalValidateTargetUri, RouteByMethod},
        namespaced::NamespacedHttpService,
        BoxHttpResponseFuture, HttpService,
    },
    uri::{
        invariant::{AbsoluteHttpUri, NormalAbsoluteHttpUri},
        HttpUri,
    },
};
use manas_repo::RepoExt;
use manas_space::SolidStorageSpace;
use name_locker::{LockKind, NameLocker};
use tower::Service;
use tracing::info;
use typed_record::{TypedRecord, TypedRecordKey};

use crate::{
    service::{
        method::{
            delete::{
                base::BaseDeleteService,
                marshaller::default::{
                    DefaultBaseDeleteResponseMarshaller, DefaultBaseDeleteResponseMarshallerConfig,
                },
            },
            get::{
                base::BaseGetService,
                marshaller::{
                    default::{
                        DefaultBaseGetResponseMarshaller, DefaultBaseGetResponseMarshallerConfig,
                    },
                    head_only::HeadOnlyBaseGetResponseMarshaller,
                },
            },
            post::{
                base::BasePostService,
                marshaller::default::{
                    DefaultBasePostResponseMarshaller, DefaultBasePostResponseMarshallerConfig,
                },
            },
            put_or_patch::{
                base::BasePutOrPatchService,
                marshaller::default::{
                    DefaultBasePutOrPatchResponseMarshaller,
                    DefaultBasePutOrPatchResponseMarshallerConfig,
                },
            },
            MethodService,
        },
        SolidStorageService, SolidStorageServiceFactory,
    },
    SolidStorage, SolidStorageExt,
};

/// Default implementation of [`SolidStorageService`].
pub struct DefaultStorageService<Storage> {
    /// Inner service.
    inner: NormalValidateTargetUri<RouteByMethod>,

    /// Storage.
    storage: Arc<Storage>,
}

impl<Storage> Clone for DefaultStorageService<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            storage: self.storage.clone(),
        }
    }
}

impl<Storage: SolidStorage> NamespacedHttpService<hyper::Body, hyper::Body>
    for DefaultStorageService<Storage>
{
    #[inline]
    fn has_in_uri_ns(&self, uri: &NormalAbsoluteHttpUri) -> bool {
        // Default implementation requires all resource uris
        // to be started with storage root resource uri.
        uri.as_str()
            .starts_with(self.storage.space().root_res_uri().as_str())
    }
}

impl<Storage: SolidStorage> SolidStorageService for DefaultStorageService<Storage> {
    type Storage = Storage;

    #[inline]
    fn storage(&self) -> &Arc<Self::Storage> {
        &self.storage
    }
}

/// A [`DefaultStorageServiceFactory`] resolves a [`DefaultStorageService`]
/// for each storage.
#[derive(Debug)]
pub struct DefaultStorageServiceFactory<Storage> {
    dev_mode: bool,
    _phantom: PhantomData<fn() -> Storage>,
}

impl<Storage> Default for DefaultStorageServiceFactory<Storage> {
    fn default() -> Self {
        Self {
            dev_mode: false,
            _phantom: PhantomData,
        }
    }
}

impl<Storage> Clone for DefaultStorageServiceFactory<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            dev_mode: self.dev_mode,
            _phantom: self._phantom,
        }
    }
}

impl<Storage: SolidStorage> DefaultStorageServiceFactory<Storage> {
    /// Create a new [`DefaultStorageServiceFactory`] with
    /// given params.
    #[inline]
    pub fn new(dev_mode: bool) -> Self {
        Self {
            dev_mode,
            ..Default::default()
        }
    }
}

impl<Storage: SolidStorage> SolidStorageServiceFactory for DefaultStorageServiceFactory<Storage> {
    type Storage = Storage;
    type Service = DefaultStorageService<Storage>;

    fn new_service(&self, storage: Arc<Storage>) -> Self::Service {
        // Construct get svc
        let get_resp_marshaller = DefaultBaseGetResponseMarshaller::new(
            storage.clone(),
            storage
                .extensions()
                .get::<DefaultBaseGetResponseMarshallerConfig>()
                .cloned()
                .unwrap_or_else(|| DefaultBaseGetResponseMarshallerConfig {
                    dev_mode: self.dev_mode,
                    ..Default::default()
                }),
        );

        let get_svc = MethodService {
            marshaller: get_resp_marshaller.clone(),
            base_method_svc: BaseGetService::new(storage.clone()),
        };

        // Head svc.
        let head_svc = MethodService {
            marshaller: HeadOnlyBaseGetResponseMarshaller::<Storage, _>::new(get_resp_marshaller),
            base_method_svc: BaseGetService::new(storage.clone()),
        };

        // Post svc.
        let post_svc = MethodService {
            base_method_svc: BasePostService::new(storage.clone()),
            marshaller: DefaultBasePostResponseMarshaller::new(
                storage.clone(),
                storage
                    .extensions()
                    .get::<DefaultBasePostResponseMarshallerConfig>()
                    .cloned()
                    .unwrap_or(DefaultBasePostResponseMarshallerConfig {
                        dev_mode: self.dev_mode,
                    }),
            ),
        };

        // Put svc.
        let put_svc = MethodService {
            base_method_svc: BasePutOrPatchService::new(storage.clone()),
            marshaller: DefaultBasePutOrPatchResponseMarshaller::new(
                storage.clone(),
                storage
                    .extensions()
                    .get::<DefaultBasePutOrPatchResponseMarshallerConfig>()
                    .cloned()
                    .unwrap_or(DefaultBasePutOrPatchResponseMarshallerConfig {
                        dev_mode: self.dev_mode,
                    }),
            ),
        };

        // Patch svc.
        let patch_svc = put_svc.clone();

        // Delete svc.
        let delete_svc = MethodService {
            base_method_svc: BaseDeleteService::new(storage.clone()),
            marshaller: DefaultBaseDeleteResponseMarshaller::new(
                storage.clone(),
                storage
                    .extensions()
                    .get::<DefaultBaseDeleteResponseMarshallerConfig>()
                    .cloned()
                    .unwrap_or(DefaultBaseDeleteResponseMarshallerConfig {
                        dev_mode: self.dev_mode,
                    }),
            ),
        };

        let mut method_services =
            HashMap::<Method, Box<dyn HttpService<hyper::Body, hyper::Body>>>::new();

        method_services.insert(Method::HEAD, Box::new(head_svc));
        method_services.insert(Method::GET, Box::new(get_svc));
        method_services.insert(Method::POST, Box::new(post_svc));
        method_services.insert(Method::PUT, Box::new(put_svc));
        method_services.insert(Method::PATCH, Box::new(patch_svc));
        method_services.insert(Method::DELETE, Box::new(delete_svc));

        let method_router = RouteByMethod::new(Arc::new(method_services));

        DefaultStorageService {
            inner: NormalValidateTargetUri::new(method_router),
            storage,
        }
    }
}

impl<Storage: SolidStorage> Service<Request<hyper::Body>> for DefaultStorageService<Storage> {
    type Response = Response<hyper::Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<hyper::Body>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "DefaultStorageService::call")]
    fn call(&mut self, mut req: Request<hyper::Body>) -> Self::Future {
        self.handle_query_params(&mut req);
        Box::pin(self.inner.call(req))
    }
}

impl<Storage: SolidStorage> DefaultStorageService<Storage> {
    /// Modify query prams as per extension config.
    fn handle_query_params(&mut self, req: &mut Request<hyper::Body>) {
        // Get req target query param mode preferred by storage.
        let preferred_req_target_query_param_mode = self
            .storage
            .extensions()
            .get_rv::<KPreferredReqTargetQueryParamMode>()
            .copied()
            .unwrap_or_default();

        // If req target uri has no query params or, mode is significant, return.
        if req.uri().query().is_none()
            || preferred_req_target_query_param_mode == ReqTargetQueryParamMode::Significant
        {
            return;
        }

        info!("Request target uri has query params. And configured mode is to treat them insignificant and strip off.");

        let req_extensions = req.extensions_mut();

        // Otherwise strip qparams from resolved resource uris.
        if let Some(res_uri) = req_extensions.get::<NormalAbsoluteHttpUri>() {
            req_extensions.insert::<NormalAbsoluteHttpUri>(
                NormalAbsoluteHttpUri::try_new_from(
                    Self::query_stripped_uri(res_uri.as_ref()).as_str(),
                )
                .expect("Must succeed"),
            );
        }
        if let Some(res_uri) = req_extensions.get::<AbsoluteHttpUri>() {
            req_extensions.insert::<AbsoluteHttpUri>(
                AbsoluteHttpUri::try_new_from(Self::query_stripped_uri(res_uri.as_ref()).as_str())
                    .expect("Must succeed"),
            );
        }
    }

    fn query_stripped_uri(uri: &HttpUri) -> String {
        format!(
            "{}://{}{}",
            uri.scheme_str(),
            uri.authority_str(),
            uri.path_str()
        )
    }
}

/// An enum representing interpretation mode of query params in request target.
/// This is supported to support databrowser like representations,
/// which use query params for custom purposes on client side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReqTargetQueryParamMode {
    /// Treat query params as significant.
    /// This is default mode.
    Significant,

    /// Treat query params insignificant, and ignore them while deducing resource uri.
    Insignificant,
    // TODO custom mode?
}

impl Default for ReqTargetQueryParamMode {
    #[inline]
    fn default() -> Self {
        Self::Significant
    }
}

/// A [`TypedRecordKey`] for recording preferred req target query param mode for a storage.
#[derive(Debug, Clone, Copy)]
pub struct KPreferredReqTargetQueryParamMode;

impl TypedRecordKey for KPreferredReqTargetQueryParamMode {
    type Value = ReqTargetQueryParamMode;
}

impl<Storage: SolidStorage> Service<()> for DefaultStorageService<Storage> {
    type Response = bool;

    type Error = Problem;

    type Future = BoxFuture<'static, Result<bool, Problem>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|_| INFALLIBLE.new_problem())
    }

    #[inline]
    fn call(&mut self, _req: ()) -> Self::Future {
        NameLocker::poll_with_lock(
            self.storage.resource_locker(),
            RepoExt::initialize(self.storage.repo()),
            Some(self.storage.space().root_res_uri().to_string()),
            LockKind::Exclusive,
        )
    }
}
