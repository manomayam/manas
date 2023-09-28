//! I define [`ODRBackendObjectPathEncodingScheme`].

use std::fmt::Debug;

use ecow::EcoString;
use manas_http::uri::component::segment::invariant::NonEmptyCleanSegmentStr;
use tower::BoxError;

use crate::object_store::object_id::normal_rootless_uri_path::NormalRootlessUriPath;

pub mod impl_;

const SLASH_CHAR: char = '/';

/// A trait for defining encoding schemes, that encode backend object paths.
///
/// `ODRObjectSpace` assigns backend agnostic uri safe object ids for it's objects.
/// But backends may offer flexibility (e.g. allowing non-ascii chars, as in fs backend), or impose further constraints over backend path.
///
/// Implementations of this trait defines an encoding scheme
/// to translate between odr-object's backend agnostic path to backend specific path in safe manner.
pub trait ODRBackendObjectPathEncodingScheme:
    Debug + Send + Sync + 'static + Sized + Unpin
{
    /// If encoding scheme is order preserving.
    ///If `scheme::encode_segment(s1).cmp(scheme::encode_segment(s2)) == s1.cmp(s2)`) holds, then scheme is order preserving.
    /// For backends, which can list entries in lexicographic order, it is recommended to always use
    /// order preserving encoding schemes.
    const IS_ORDER_PRESERVING: bool;

    /// Type of encode error.
    type EncodeError: Into<BoxError> + Debug + Send + Sync + 'static + Sized;

    /// Type of decode error.
    type DecodeError: Into<BoxError> + Debug + Send + Sync + 'static + Sized;

    /// Encode given normal and clean uri path segment to backend obj path segment.
    ///
    fn encode_segment(
        odr_obj_path_segment: &NonEmptyCleanSegmentStr,
    ) -> Result<EcoString, Self::EncodeError>;

    /// Decode clean uri path segment from given backend path segment .
    fn decode_segment(
        backend_obj_path_segment: &str,
    ) -> Result<NonEmptyCleanSegmentStr, Self::DecodeError>;
}

mod seal {
    use super::ODRBackendObjectPathEncodingScheme;

    // Seal.
    pub trait Sealed {}

    impl<ES: ODRBackendObjectPathEncodingScheme> Sealed for ES {}
}

/// An extension trait for types implementing [`ODRBackendObjectPathEncodingScheme`].
pub trait ODRBackendObjectPathEncodingSchemeExt:
    ODRBackendObjectPathEncodingScheme + seal::Sealed
{
    /// Encode root relative odr object path into backend path as per scheme.
    fn encode(
        root_relative_odr_obj_path: &NormalRootlessUriPath,
    ) -> Result<String, ODRBackendObjectPathEncodeError<Self>>;

    /// Decode backend obj path into root relative odr obj path as per scheme.
    fn decode(
        backend_obj_path: &str,
    ) -> Result<NormalRootlessUriPath<'static>, ODRBackendObjectPathDecodeError<Self>>;
}

impl<ES: ODRBackendObjectPathEncodingScheme> ODRBackendObjectPathEncodingSchemeExt for ES {
    fn encode(
        root_relative_odr_obj_path: &NormalRootlessUriPath,
    ) -> Result<String, ODRBackendObjectPathEncodeError<Self>> {
        let mut backend_obj_path = String::new();

        // For each obj path uri segment,
        for (i, odr_obj_path_segment) in root_relative_odr_obj_path.split(SLASH_CHAR).enumerate() {
            let backend_obj_path_segment = if odr_obj_path_segment.is_empty() {
                // Empty segment will always maps to empty segment.
                EcoString::new()
            } else {
                // Else encode with scheme.
                let odr_obj_path_segment =
                    NonEmptyCleanSegmentStr::try_new_from(odr_obj_path_segment)
                        .expect("Must be valid non empty clean segment, ");

                Self::encode_segment(&odr_obj_path_segment).map_err(|e| {
                    ODRBackendObjectPathEncodeError::ODRObjectPathHasInvalidSegment {
                        segment_index: i,
                        error: e,
                    }
                })?
            };

            // Push slash join delim.
            if i > 0 {
                // TODO should use join over fallible iterator instead.
                backend_obj_path.push(SLASH_CHAR);
            }
            backend_obj_path.push_str(&backend_obj_path_segment);
        }

        Ok(backend_obj_path)
    }

    fn decode(
        backend_obj_path: &str,
    ) -> Result<NormalRootlessUriPath<'static>, ODRBackendObjectPathDecodeError<Self>> {
        let mut odr_obj_path = String::new();

        // For each backend obj path segment,
        for (i, backend_obj_path_segment) in backend_obj_path.split(SLASH_CHAR).enumerate() {
            let odr_obj_path_segment = if backend_obj_path_segment.is_empty() {
                // Empty segment will always maps to empty segment.
                EcoString::new()
            } else {
                // Else encode with scheme.
                Self::decode_segment(backend_obj_path_segment)
                    .map_err(|e| {
                        ODRBackendObjectPathDecodeError::BackendObjectPathHasInvalidSegment {
                            segment_index: i,
                            error: e,
                        }
                    })?
                    .into_subject()
                    .into()
            };

            // Push slash join delim.
            if i > 0 {
                // TODO should use join over fallible iterator instead.
                odr_obj_path.push(SLASH_CHAR);
            }
            odr_obj_path.push_str(&odr_obj_path_segment);
        }

        // SAFETY: Must be valid normal path, as concatenated from normal and clean segments.
        // And must be rootless, due to join op.
        Ok(unsafe { NormalRootlessUriPath::new_unchecked(odr_obj_path.into()) })
    }
}

/// Type for error in backend object path encoding.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ODRBackendObjectPathEncodeError<ODRBackendObjPathES>
where
    ODRBackendObjPathES: ODRBackendObjectPathEncodingScheme,
{
    /// ODR object path has invalid path segment.
    #[error("ODR object path has invalid path segment.")]
    ODRObjectPathHasInvalidSegment {
        /// Index of invalid segment.
        segment_index: usize,

        /// Error about segment.
        error: ODRBackendObjPathES::EncodeError,
    },
}

/// Type for error in backend object path decoding.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ODRBackendObjectPathDecodeError<ODRBackendObjPathES>
where
    ODRBackendObjPathES: ODRBackendObjectPathEncodingScheme,
{
    /// Obj backend path has invalid path segment.
    #[error("Backend object path has invalid path segment.")]
    BackendObjectPathHasInvalidSegment {
        /// Index of invalid segment.
        segment_index: usize,

        /// Error about segment.
        error: ODRBackendObjPathES::DecodeError,
    },
}

#[cfg(test)]
pub mod mock {
    use super::impl_::pct_decoded::{mock::MockDPrefix, PctDecodedBackendObjectPathEncodingScheme};

    /// Type alias for mock obj backend path encoding scheme.
    pub type MockBackendObjectPathEncodingScheme =
        PctDecodedBackendObjectPathEncodingScheme<MockDPrefix>;
}

#[cfg(test)]
mod tests {
    use claims::assert_matches;
    use impl_::pct_decoded::PctDecodedBackendObjectPathEncodeError;
    use rstest::*;

    use super::{mock::MockBackendObjectPathEncodingScheme, *};

    #[rstest]
    #[case("", Ok(""))]
    #[case("abc/def.png", Ok("abc/def.png"))]
    #[case("abc/def/", Ok("abc/def/"))]
    #[case("abc/def%20bcd", Ok("abc/def bcd"))]
    #[case("ramayana/%E0%A4%B0%E0%A4%BE%E0%A4%AE", Ok("ramayana/राम"))]
    #[case("a%2Fb/b%20cd", Ok("_$$_a%2Fb/b cd"))]
    #[case("ab%24c/de%2Ff/", Ok("_$$_ab%24c/_$$_de%2Ff/"))]
    #[case(
        "abc/def_$$_",
        Err(ODRBackendObjectPathEncodeError::ODRObjectPathHasInvalidSegment { segment_index: 1, error: PctDecodedBackendObjectPathEncodeError::UriSegmentHasExtraEncodingSemantics })
    )]
    #[case("abc/def", Ok("abc/def"))]
    fn encode_works_correctly(
        #[case] root_relative_odr_obj_path: &str,
        #[case] expectation_hint: Result<
            &str,
            ODRBackendObjectPathEncodeError<MockBackendObjectPathEncodingScheme>,
        >,
    ) {
        // SAFETY: claimed valid root relative obj path.
        let root_relative_odr_obj_path =
            unsafe { NormalRootlessUriPath::new_unchecked(root_relative_odr_obj_path.into()) };

        let encode_result =
            MockBackendObjectPathEncodingScheme::encode(&root_relative_odr_obj_path);

        assert_eq!(
            encode_result,
            expectation_hint.map(|backend_obj_path| backend_obj_path.to_owned()),
            "Encode expectation not satisfied."
        );
    }

    #[rstest]
    #[case("", "")]
    #[case("abc/def.png", "abc/def.png")]
    #[case("abc/def/", "abc/def/")]
    #[case("abc/def bcd", "abc/def%20bcd")]
    #[case("ramayana/राम", "ramayana/%E0%A4%B0%E0%A4%BE%E0%A4%AE")]
    #[case("_$$_a%2Fb/b cd", "a%2Fb/b%20cd")]
    #[case("_$$_ab%24c/_$$_de%2Ff/", "ab%24c/de%2Ff/")]

    fn decode_works_correctly_for_valid_backend_path(
        #[case] backend_obj_path: &str,
        #[case] expected_path_str: &str,
    ) {
        let decoded_odr_path = claims::assert_ok!(
            MockBackendObjectPathEncodingScheme::decode(backend_obj_path),
            "Decoding backend path erred for valid input"
        );

        assert_eq!(
            decoded_odr_path.as_ref(),
            expected_path_str,
            "Decode expectation not satisfied."
        );
    }

    #[rstest]
    #[case("abc/_$$__abc def", 1)]
    #[case("abc/_$$__abc%2fdef", 1)]
    #[case("abc/cd/..", 2)]
    fn decode_raises_error_for_path_with_invalid_segment(
        #[case] backend_obj_path: &str,
        #[case] expected_invalid_segment_index: usize,
    ) {
        let decode_error = claims::assert_err!(
            MockBackendObjectPathEncodingScheme::decode(backend_obj_path),
            "Decoding backend path succeeded for invalid input"
        );

        assert_matches!(
            decode_error,
            ODRBackendObjectPathDecodeError::BackendObjectPathHasInvalidSegment {
                segment_index: invalid_segment_index,
                error: _
            } if invalid_segment_index == expected_invalid_segment_index
        );
    }
}
