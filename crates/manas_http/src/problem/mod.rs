//! I provide utilities to deal with http problems.
//!

use http::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    HeaderValue, Response, StatusCode,
};
use http_api_problem::{ApiError, HttpApiProblem, PROBLEM_JSON_MEDIA_TYPE};
use seal::Sealed;

use crate::body::Body;

mod seal {
    use http_api_problem::{ApiError, HttpApiProblem};

    pub trait Sealed {}
    impl Sealed for HttpApiProblem {}
    impl Sealed for ApiError {}
}

/// An extension trait for [`HttpApiProblem`].
pub trait HttpApiProblemExt: Sealed {
    /// Convert problem to http response.
    fn to_http_response(&self) -> Response<Body>;
}

impl HttpApiProblemExt for HttpApiProblem {
    fn to_http_response(&self) -> Response<Body> {
        let json = self.json_bytes();
        let length = json.len() as u64;

        let (mut parts, body) = Response::new(json.into()).into_parts();

        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(PROBLEM_JSON_MEDIA_TYPE),
        );
        parts.headers.insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(&length.to_string()).unwrap(),
        );
        parts.status = self
            .status
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            .as_u16()
            .try_into()
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        Response::from_parts(parts, body)
    }
}

/// An extension trait for [`ApiError`].
pub trait ApiErrorExt {
    /// Convert error into http response.
    fn into_http_response(self) -> Response<Body>;
}

impl ApiErrorExt for ApiError {
    fn into_http_response(self) -> Response<Body> {
        let problem = self.into_http_api_problem();
        problem.to_http_response()
    }
}
