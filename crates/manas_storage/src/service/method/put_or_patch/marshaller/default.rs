//! I define default implementation of [`BaseResponseMarshaller`](super::super::super::BaseResponseMarshaller)
//! for marshalling [`BasePutOrPatchService`](super::super::base::BasePutOrPatchService) responses.
//!

use std::{convert::Infallible, sync::Arc, task::Poll};

use headers::{ETag, HeaderMapExt};
use http::{Response, StatusCode};
use http_api_problem::ApiError;
use hyper::Body;
use manas_http::{representation::metadata::KDerivedETag, service::BoxHttpResponseFuture};
use tower::Service;
use typed_record::{TypedRecord, TypedRecordKey};

use crate::{
    service::method::{
        common::snippet::authorization::attach_authorization_context,
        put_or_patch::base::BasePutOrPatchResponse,
    },
    SolidStorage,
};

/// Config for[`DefaultBasePutOrPatchResponseMarshaller`].
#[derive(Debug, Clone, Default)]
pub struct DefaultBasePutOrPatchResponseMarshallerConfig {
    /// If dev mode is enabled.
    pub dev_mode: bool,
}

/// Default implementation of [`BaseResponseMarshaller`](super::super::super::BaseResponseMarshaller)
/// for marshalling [`BasePutOrPatchService`](super::super::base::BasePutOrPatchService) responses.
#[derive(Debug)]
pub struct DefaultBasePutOrPatchResponseMarshaller<Storage> {
    /// Storage.
    pub storage: Arc<Storage>,

    /// Marshal config.
    pub marshal_config: DefaultBasePutOrPatchResponseMarshallerConfig,
}

impl<Storage> Clone for DefaultBasePutOrPatchResponseMarshaller<Storage> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            marshal_config: self.marshal_config.clone(),
        }
    }
}

impl<Storage> DefaultBasePutOrPatchResponseMarshaller<Storage> {
    /// Get new [`DefaultBasePutOrPatchResponseMarshaller`].
    #[inline]
    pub fn new(
        storage: Arc<Storage>,
        marshal_config: DefaultBasePutOrPatchResponseMarshallerConfig,
    ) -> Self {
        Self {
            storage,
            marshal_config,
        }
    }
}

impl<Storage> Service<Result<BasePutOrPatchResponse<Storage>, ApiError>>
    for DefaultBasePutOrPatchResponseMarshaller<Storage>
where
    Storage: SolidStorage,
{
    type Response = Response<Body>;

    type Error = Infallible;

    type Future = BoxHttpResponseFuture<Body>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    #[tracing::instrument(skip_all, name = "DefaultBasePutOrPatchResponseMarshaller::call")]
    fn call(&mut self, req: Result<BasePutOrPatchResponse<Storage>, ApiError>) -> Self::Future {
        Box::pin(futures::future::ready(Ok(match req {
            Ok(resp) => self.marshal_ok(resp),
            Err(err) => self.marshal_err(err),
        })))
    }
}

impl<Storage> DefaultBasePutOrPatchResponseMarshaller<Storage>
where
    Storage: SolidStorage,
{
    /// Marshal `BasePutOrPatch` success response.
    #[allow(clippy::vec_init_then_push)]
    pub fn marshal_ok(&self, response: BasePutOrPatchResponse<Storage>) -> Response<Body> {
        // Create builder.
        let mut builder = Response::builder();

        builder = builder.status(if response.is_created {
            // If resource is newly created.
            StatusCode::CREATED
        } else {
            // If resource is updated.
            StatusCode::OK
        });

        // Set  etag, if available.
        if let Some(etag) = response
            .new_rep_validators
            .and_then(|validators| validators.get_rv::<KDerivedETag>().cloned())
        {
            builder
                .headers_mut()
                .expect("Must be ik")
                .typed_insert::<ETag>(etag.into());
        };

        // Set empty body, and return response.
        builder
            .body(Body::empty())
            .expect("Must be valid hyper response.")
    }

    /// Marshal `BasePutOrPatch` error.
    pub fn marshal_err(&self, mut error: ApiError) -> Response<Body> {
        if self.marshal_config.dev_mode {
            attach_authorization_context::<Storage>(&mut error);

            if let Some(patch_error_context) = error
                .extensions_mut()
                .remove_rec_item::<KPatchErrorContext>()
            {
                error.add_field("patch_error_context", patch_error_context);
            }
        }

        error.into_hyper_response()
    }
}

/// A typed record key for patch error context.
#[derive(Debug, Clone)]
pub struct KPatchErrorContext;

impl TypedRecordKey for KPatchErrorContext {
    type Value = String;
}
