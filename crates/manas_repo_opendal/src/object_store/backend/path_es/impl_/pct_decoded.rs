//! I define [`PctDecodedBackendObjectPathEncodingScheme`] An
//! implementation of [`ODRBackendObjectPathEncodingScheme`],
//! that encodes a normal uri path segment to it's pct
//! decoded version.
//!

use std::{marker::PhantomData, ops::Deref};

use ecow::EcoString;
use gdp_rs::{
    predicate::impl_::all_of::{IntoPL, SculptPL},
    Proven,
};
use manas_http::uri::component::segment::{
    invariant::NonEmptyCleanSegmentStr,
    predicate::{
        is_non_dot::IsNonDot, is_non_empty::IsNonEmpty, is_normal::PctEncodingNormalization,
    },
    safe_token::TSegmentSafeToken,
};
use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use regex::Regex;
use tower::BoxError;

use crate::object_store::backend::path_es::ODRBackendObjectPathEncodingScheme;

/// An implementation of [`ODRBackendObjectPathEncodingScheme`], that
/// encodes a odr object path segment to it's pct decoded backend version.
///
/// When odr-path-segment contains reserved chars (like slash) in pct encoded form, then segment is not representable on pct decoded backend form.
/// And when segment contains sub delims, then pct decoded form will cause conflicts,
/// as uri normalization allows them to be in both encoded and decoded forms with out being equivalent.,
///
/// In both these special cases, scheme fallbacks to identical scheme with a reserved discriminant prefix prepended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PctDecodedBackendObjectPathEncodingScheme<DPrefix = DefaultDPrefix>
where
    DPrefix: TSegmentSafeToken,
{
    _phantom: PhantomData<DPrefix>,
}

/// matches path reserved chars, and sub delims.
///
/// reserved: "/"
/// sub-delims  = "!" / "$" / "&" / "'" / "(" / ")"
///                       / "*" / "+" / "," / ";" / "="
///
/// TODO should take reserved chars as type param.
static SPECIAL_CHAR_PCT_FORM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("%(2F|21|24|26|27|28|29|2A|2B|2C|3B|3D)").expect("Must be valid"));

impl<DPrefix> ODRBackendObjectPathEncodingScheme
    for PctDecodedBackendObjectPathEncodingScheme<DPrefix>
where
    DPrefix: TSegmentSafeToken,
{
    const IS_ORDER_PRESERVING: bool = false;

    type EncodeError = PctDecodedBackendObjectPathEncodeError;

    type DecodeError = BoxError;

    fn encode_segment(
        odr_obj_path_segment: &NonEmptyCleanSegmentStr,
    ) -> Result<EcoString, Self::EncodeError> {
        // Ensure reserved discriminant prefix is not present.
        // TODO limit check to just not being prefix?
        if odr_obj_path_segment.contains(DPrefix::token().as_str()) {
            return Err(
                PctDecodedBackendObjectPathEncodeError::UriSegmentHasExtraEncodingSemantics,
            );
        }

        // If segment contains reserved chars, or sub delims
        // in pct encoded form, then fallback to identity encoding with attached discriminant.
        if SPECIAL_CHAR_PCT_FORM_REGEX.is_match(odr_obj_path_segment.as_ref()) {
            Ok(format!(
                "{}{}",
                DPrefix::token().as_str(),
                odr_obj_path_segment.as_str()
            )
            .into())
        } else {
            Ok(percent_decode_str(odr_obj_path_segment.as_str())
                .decode_utf8_lossy()
                .as_ref()
                .into())
        }
    }

    fn decode_segment(
        backend_obj_path_segment: &str,
    ) -> Result<NonEmptyCleanSegmentStr, Self::DecodeError> {
        let dprefix_str = DPrefix::token().as_str();

        // If has discriminant suffix, then strip and return remaining as it is.
        if let Some(dprefix_stripped_segment) = backend_obj_path_segment.strip_prefix(dprefix_str) {
            Ok(NonEmptyCleanSegmentStr::try_new_from(
                dprefix_stripped_segment,
            )?)
        } else {
            // Encode, normalize and return.
            let normal_segent = Proven::void_proven(backend_obj_path_segment)
                .infer::<PctEncodingNormalization<_>>(Default::default())
                .infer::<IntoPL<_, _>>(Default::default());
            Ok(normal_segent
                .try_extend_predicate::<IsNonEmpty>()?
                .try_extend_predicate::<IsNonDot>()?
                .infer::<SculptPL<_, _, _, _>>(Default::default()))
        }
    }
}

///An error type for errorsin encoding obj backend path with pct scheme.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum PctDecodedBackendObjectPathEncodeError {
    /// Uri segment has extra encoding semantics.
    #[error("Uri segment has extra encoding semantics.")]
    UriSegmentHasExtraEncodingSemantics,
}

/// Default [`TSegmentSafeToken`] to be used as discriminant prefix
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultDPrefix;

static DEFAULT_DPREFIX_TOKEN: Lazy<NonEmptyCleanSegmentStr> =
    Lazy::new(|| NonEmptyCleanSegmentStr::try_new_from("_$$_").expect("Must be valid"));

impl TSegmentSafeToken for DefaultDPrefix {
    #[inline]
    fn token() -> &'static NonEmptyCleanSegmentStr {
        DEFAULT_DPREFIX_TOKEN.deref()
    }
}

#[cfg(test)]
pub mod mock {

    use super::*;

    /// A mock [`Delim`] to be used as discriminant prefix
    pub type MockDPrefix = DefaultDPrefix;
}

#[cfg(test)]
mod tests {
    use claims::assert_err;
    use rstest::*;

    use super::{mock::*, *};

    #[rstest]
    #[case::simple("abc.def", Ok("abc.def"))]
    #[case::with_normal_sub_delim("abc$def;cd!", Ok("abc$def;cd!"))]
    #[case::pct_encoded_char("abc%20def", Ok("abc def"))]
    #[case::pct_encoded_non_ascii("%E0%A4%B0%E0%A4%BE%E0%A4%AE", Ok("राम"))]
    #[case::slash_pct_encoded("abc%2Fdef", Ok("_$$_abc%2Fdef"))]
    #[case::sub_delim_pct_encoded("abc%20def%24", Ok("_$$_abc%20def%24"))]
    #[case::with_dprefix(
        "_$$_abc",
        Err(PctDecodedBackendObjectPathEncodeError::UriSegmentHasExtraEncodingSemantics)
    )]
    fn encode_segment_woks_correctly(
        #[case] uri_path_segment_str: &str,
        #[case] expected_result_hint: Result<&str, PctDecodedBackendObjectPathEncodeError>,
    ) {
        let result = PctDecodedBackendObjectPathEncodingScheme::<MockDPrefix>::encode_segment(
            &NonEmptyCleanSegmentStr::try_new_from(uri_path_segment_str)
                .expect("Claimed valid uri path segment str."),
        );

        assert_eq!(
            result.map(|backend_path_segment| backend_path_segment.as_str().to_owned()),
            expected_result_hint.map(|expected_str| expected_str.to_owned()),
            "encode_segment expectation failed."
        );
    }

    #[rstest]
    #[case::simple("abc.def", "abc.def")]
    #[case::with_normal_sub_delim("abc$def;cd!", "abc$def;cd!")]
    #[case::special_ascii_char("abc def", "abc%20def")]
    #[case::non_ascii("राम", "%E0%A4%B0%E0%A4%BE%E0%A4%AE")]
    #[case::fallback_scheme_encoded1("_$$_abc%2Fdef", "abc%2Fdef")]
    #[case::fallback_scheme_encoded2("_$$_abc%20def%24", "abc%20def%24")]
    fn decode_segment_woks_correctly_for_valid_segment_str(
        #[case] backend_path_segment_str: &str,
        #[case] expected_result_hint: &str,
    ) {
        let decoded_segment = claims::assert_ok!(
            PctDecodedBackendObjectPathEncodingScheme::<MockDPrefix>::decode_segment(
                backend_path_segment_str,
            ),
            "decode_segment raises error for valid ssegment str"
        );

        assert_eq!(decoded_segment.as_str(), expected_result_hint);
    }

    #[rstest]
    #[case::invalid_fallback_encoded("_$$_abc def")]
    #[case::invalid_fallback_encoded2("_$$_abc%2fdef")]
    #[case::non_clean("..")]
    fn decode_segment_raises_error_for_invalid_input(#[case] backend_path_segment_str: &str) {
        let result = PctDecodedBackendObjectPathEncodingScheme::<MockDPrefix>::decode_segment(
            backend_path_segment_str,
        );

        assert_err!(
            result,
            "decode_segment successfully decodes invalid backed segment str."
        );
    }
}
