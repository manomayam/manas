//! I define an implementation of
//! [`ODRBackendObjectPathEncodingScheme`] with identity mapping.
//!

use std::convert::Infallible;

use ecow::EcoString;
use manas_http::uri::component::segment::invariant::NonEmptyCleanSegmentStr;
use tower::BoxError;

use crate::object_store::backend::path_es::ODRBackendObjectPathEncodingScheme;

/// An implementation of [`ODRBackendObjectPathEncodingScheme`], that
/// encodes an uri path segment to it's identical backend segment
/// with out any further encoding.
#[derive(Debug, Clone)]
pub struct IdenticalBackendObjectPathEncodingScheme;

impl ODRBackendObjectPathEncodingScheme for IdenticalBackendObjectPathEncodingScheme {
    const IS_ORDER_PRESERVING: bool = true;

    type EncodeError = Infallible;

    type DecodeError = BoxError;

    fn encode_segment(
        odr_obj_path_segment: &NonEmptyCleanSegmentStr,
    ) -> Result<EcoString, Self::EncodeError> {
        Ok(EcoString::from(odr_obj_path_segment.as_ref().as_ref()))
    }

    #[inline]
    fn decode_segment(
        backend_obj_path_segment: &str,
    ) -> Result<NonEmptyCleanSegmentStr, Self::DecodeError> {
        Ok(NonEmptyCleanSegmentStr::try_new_from(
            backend_obj_path_segment,
        )?)
    }
}
