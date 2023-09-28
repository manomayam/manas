//! I define [`Prefer`] typed header and related structures.
//!
use headers::{Header, HeaderName};
use tracing::error;
use vec1::smallvec_v1::SmallVec1;

use crate::field::rules::flat_csv::{Comma, FlatCsv};

mod preference;

pub use preference::*;

/// `Prefer` header is defined in [`rfc7240`](https://datatracker.ietf.org/doc/html/rfc7240#section-1)
///
/// The Prefer request header field is used to indicate that particular
/// server behaviors are preferred by the client but are not required for
/// successful completion of the request.  Prefer is similar in nature to
/// the Expect header field defined by Section 6.1.2 of RFC7231 with
/// the exception that servers are allowed to ignore stated preferences.
///```txt
/// ABNF:
///
///     Prefer     = "Prefer" ":" 1#preference
///     preference = token [ BWS "=" BWS word ]
///                 *( OWS ";" [ OWS parameter ] )
///     parameter  = token [ BWS "=" BWS word ]
/// ```
#[derive(Debug, Clone)]
pub struct Prefer {
    /// List of one or more preferences.
    pub preferences: SmallVec1<[Preference; 1]>,
}

/// Constant for `prefer` header-name.
pub static PREFER: HeaderName = HeaderName::from_static("prefer");

impl Header for Prefer {
    #[inline]
    fn name() -> &'static HeaderName {
        &PREFER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        Ok(Prefer {
            preferences: SmallVec1::try_from_smallvec(
                values
                    .flat_map(|value| FlatCsv::<Comma>::from(value).iter())
                    .map(|value_str| value_str.parse())
                    .collect::<Result<_, _>>()
                    .map_err(|_| headers::Error::invalid())?,
            )
            .map_err(|_| {
                error!("Zero number of preferences not allowed");
                headers::Error::invalid()
            })?,
        })
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(self.preferences.iter().map(|preference| {
            preference
                .str_encode()
                .parse()
                .expect("Preference is always a valid HeaderValue")
        }));
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use claims::*;
    use headers::HeaderValue;
    use preference::tests_decode::assert_valid_preference;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case("Lenient, abc/def; a=1")]
    #[case("abc def; b=2; Lenient")]
    #[case("abc = def/pqr; b=2")]
    #[case("abc = def pqr; b=2")]
    #[case::invalid_param_key("pref1; abc; def/ghi = 123")]
    fn header_with_invalid_preferences_will_be_rejected(#[case] header_value_str: &str) {
        let header_value = assert_ok!(HeaderValue::from_str(header_value_str));
        assert_err!(Prefer::decode(&mut std::iter::once(&header_value)));
    }

    fn assert_correspondence<'a>(
        header_value_strs: impl Iterator<Item = &'a str>,
        prefer: &Prefer,
    ) {
        let header_values = header_value_strs.collect::<Vec<_>>();
        assert_eq!(
            prefer.preferences.len(),
            header_values.len(),
            "Mismatched length"
        );

        for (i, preference) in prefer.preferences.iter().enumerate() {
            let preference2 = assert_ok!(Preference::from_str(header_values[i]));
            assert_eq!(preference, &preference2, "Mismatched preference");
        }
    }

    #[rstest]
    #[case(&["foo; bar", "return=minimal; foo=\"some parameter\""])]
    #[case(&[r#"return=representation; include="http://www.w3.org/ns/ldp#PreferMembership http://www.w3.org/ns/ldp#PreferMinimalContainer""#,"respond-async; wait=100"])]
    fn valid_headers_will_be_decoded_correctly(#[case] header_value_strs: &[&str]) {
        let header_values: Vec<HeaderValue> = header_value_strs
            .iter()
            .map(|v| assert_ok!(HeaderValue::from_str(v)))
            .collect();
        let prefer = assert_ok!(Prefer::decode(&mut header_values.iter()));
        assert_correspondence(header_value_strs.iter().copied(), &prefer);
    }

    #[rstest]
    #[case(&["foo; bar", "return=minimal; foo=\"some parameter\""])]
    #[case(&[r#"return=representation; include="http://www.w3.org/ns/ldp#PreferMembership http://www.w3.org/ns/ldp#PreferMinimalContainer""#,"respond-async; wait=100"])]
    fn encode_works_correctly(#[case] preference_strs: &[&str]) {
        let prefer = Prefer {
            preferences: assert_ok!(SmallVec1::try_from_smallvec(
                preference_strs
                    .iter()
                    .map(|s| assert_valid_preference(s))
                    .collect()
            )),
        };

        let mut header_values = Vec::new();
        prefer.encode(&mut header_values);

        assert_correspondence(header_values.iter().map(|v| v.to_str().unwrap()), &prefer);
    }
}
