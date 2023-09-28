//! I define few code snippets to handle request headers.
//!

use std::borrow::Cow;

use dyn_problem::Problem;
use headers::{ContentLength, ContentType, HeaderMap, HeaderMapExt};
use http::{
    header::{IF_MATCH, IF_MODIFIED_SINCE, IF_NONE_MATCH, IF_RANGE, IF_UNMODIFIED_SINCE},
    StatusCode,
};
use http_api_problem::ApiError;
use hyper::body::SizeHint;
use manas_http::{
    header::common::media_type::MediaType, representation::metadata::derived_etag::DerivedETag,
};
use manas_repo::service::resource_operator::common::preconditions::{
    impl_::http::HttpPreconditionsEvalResult, KPreconditionsEvalResult,
};
use manas_specs::{
    protocol::{REQ_CLIENT_CONTENT_TYPE, REQ_SERVER_CONTENT_TYPE},
    SpecProblem,
};
use tracing::debug;
use typed_record::TypedRecord;

/// Get new header map composed of conditional headers with
/// base normalized etags.
// TODO tests
#[tracing::instrument(skip_all)]
pub fn etag_base_normalized_conditional_headers(headers: &HeaderMap) -> HeaderMap {
    let mut normalized_conditional_headers = HeaderMap::new();

    for h_name in [IF_MATCH, IF_NONE_MATCH, IF_RANGE] {
        // Get normalized header values.
        let values = headers
            .get_all(h_name.clone())
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|h_value| {
                // If it is an non-any ETagRange:
                // rfc: A valid entity-tag can be
                // distinguished from a valid HTTP-date by
                // examining the first three characters for a
                // DQUOTE.
                let normalized_h_value =
                    if h_value.starts_with('"') || h_value.starts_with(r#"W/""#) {
                        DerivedETag::base_normalize_etag_range(h_value)
                    }
                    // Otherwise original value.
                    else {
                        Cow::Borrowed(h_value)
                    }
                    .parse()
                    .expect("Must be valid header value");

                debug!(
                    "h_name: {}, h_value: {}, normalized_h_value: {:?}",
                    h_name, h_value, normalized_h_value
                );
                normalized_h_value
            });

        for value in values {
            normalized_conditional_headers.append(h_name.clone(), value);
        }
    }

    for h_name in [IF_MODIFIED_SINCE, IF_UNMODIFIED_SINCE].iter() {
        for value in headers.get_all(h_name).iter() {
            normalized_conditional_headers.append(h_name.clone(), value.clone());
        }
    }
    normalized_conditional_headers
}

/// Resolve content-type from request headers.
/// If is not valid, returns appropriate spec problem.
///
/// Req: Clients MUST use the Content-Type HTTP header in
/// PUT, POST and PATCH requests.
///
/// Req: Server MUST reject PUT, POST and PATCH requests
/// without the Content-Type header with a status code of 400.
#[allow(clippy::result_large_err)]
pub fn resolve_req_content_type(headers: &HeaderMap) -> Result<MediaType, ApiError> {
    headers
        .typed_get::<ContentType>()
        .and_then(|content_type| MediaType::try_from(content_type).ok())
        .ok_or_else::<ApiError, _>(|| {
            SpecProblem::new(StatusCode::BAD_REQUEST)
                .with_violated(&REQ_CLIENT_CONTENT_TYPE)
                .with_recourse_as_per(&REQ_SERVER_CONTENT_TYPE)
                .into()
        })
}

/// Resolve content-length-hint from request headers.
///
/// It sets to exact hint, if `Content-Length` header exists.
/// Else it sets default hint.
pub fn resolve_req_content_length_hint(headers: &HeaderMap) -> SizeHint {
    headers
        .typed_get::<ContentLength>()
        .map(|content_length| SizeHint::with_exact(content_length.0))
        .unwrap_or_default()
}

/// Resolve preconditions evaluated status code, if any.
pub fn resolve_preconditions_eval_status(problem: &Problem) -> Option<StatusCode> {
    problem
        .extensions()
        .get_rv::<KPreconditionsEvalResult>()
        .and_then(|r| r.as_any().downcast_ref::<HttpPreconditionsEvalResult>())
        .and_then(|r| r.as_return().cloned())
}
