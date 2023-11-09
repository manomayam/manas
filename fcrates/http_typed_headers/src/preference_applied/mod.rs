//! I define [`PreferenceApplied`] typed header and related structures.
//!
use headers::{Header, HeaderName};
use tracing::error;
use vec1::smallvec_v1::SmallVec1;

use crate::common::field::rules::flat_csv::{Comma, FlatCsv};

mod applied_pref;

pub use applied_pref::*;

/// `Prefer` header is defined in [`rfc7240`](https://datatracker.ietf.org/doc/html/rfc7240#section-1)
///
/// The Prefer request header field is used to indicate that particular
/// server behaviors are preferred by the client but are not required for
/// successful completion of the request.  Prefer is similar in nature to
/// the Expect header field defined by Section 6.1.2 of RFC7231 with
/// the exception that servers are allowed to ignore stated applied_prefs.
///```txt
/// ABNF:
///
///     Prefer     = "Prefer" ":" 1#applied_pref
///     applied_pref = token [ BWS "=" BWS word ]
///                 *( OWS ";" [ OWS parameter ] )
///     parameter  = token [ BWS "=" BWS word ]
/// ```
#[derive(Debug, Clone)]
pub struct PreferenceApplied {
    /// List of one or more applied-prefs.
    pub applied_prefs: SmallVec1<[AppliedPref; 1]>,
}

/// Static for `preference-applied` header-name.
pub static PREFERENCE_APPLIED: HeaderName = HeaderName::from_static("preference-applied");

impl Header for PreferenceApplied {
    #[inline]
    fn name() -> &'static HeaderName {
        &PREFERENCE_APPLIED
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        Ok(PreferenceApplied {
            applied_prefs: SmallVec1::try_from_smallvec(
                values
                    .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
                    .map(|value_str| value_str.parse())
                    .collect::<Result<_, _>>()
                    .map_err(|_| headers::Error::invalid())?,
            )
            .map_err(|_| {
                error!("Zero number of applied preferences not allowed");
                headers::Error::invalid()
            })?,
        })
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(self.applied_prefs.iter().map(|applied_pref| {
            applied_pref
                .str_encode()
                .parse()
                .expect("AppliedPref is always a valid HeaderValue")
        }));
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use applied_pref::tests_decode::assert_valid_applied_pref;
    use claims::*;
    use headers::HeaderValue;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("Lenient, abc/def")]
    #[case("def; b=2")]
    #[case("abc = def/pqr")]
    #[case("abc = def pqr")]
    fn header_with_invalid_applied_prefs_will_be_rejected(#[case] header_value_str: &str) {
        assert_err!(PreferenceApplied::decode(&mut std::iter::once(
            &header_value_str.parse().unwrap()
        )));
    }

    fn assert_correspondence<'a>(
        header_value_strs: impl Iterator<Item = &'a str>,
        preference_applied: &PreferenceApplied,
    ) {
        let header_value_strs: Vec<_> = header_value_strs.collect();
        assert_eq!(
            preference_applied.applied_prefs.len(),
            header_value_strs.len(),
            "Mismatched length"
        );

        for (i, applied_pref) in preference_applied.applied_prefs.iter().enumerate() {
            let applied_pref2 = assert_ok!(AppliedPref::from_str(header_value_strs[i]));
            assert_eq!(applied_pref, &applied_pref2, "Mismatched applied_pref");
        }
    }

    #[rstest]
    #[case(&["foo", "return=minimal"])]
    #[case(&[r#"return=representation"#,"respond-async"])]
    fn valid_headers_will_be_decoded_correctly(#[case] header_value_strs: &[&str]) {
        let header_values: Vec<HeaderValue> = header_value_strs
            .iter()
            .map(|v| assert_ok!(HeaderValue::from_str(v)))
            .collect();
        let preference_applied = assert_ok!(PreferenceApplied::decode(&mut header_values.iter()));
        assert_correspondence(header_value_strs.iter().copied(), &preference_applied);
    }

    #[rstest]
    #[case(&["foo", "return=minimal"])]
    #[case(&[r#"return=representation"#,"respond-async"])]
    fn encode_works_correctly(#[case] applied_pref_strs: &[&str]) {
        let preference_applied = PreferenceApplied {
            applied_prefs: assert_ok!(SmallVec1::try_from_smallvec(
                applied_pref_strs
                    .iter()
                    .map(|s| assert_valid_applied_pref(s))
                    .collect()
            )),
        };
        let mut header_values = Vec::new();
        preference_applied.encode(&mut header_values);

        assert_correspondence(
            header_values.iter().map(|v| v.to_str().unwrap()),
            &preference_applied,
        );
    }
}
