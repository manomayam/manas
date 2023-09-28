//! I define [`Slug`] typed header and related structures.
//!

use std::{borrow::Cow, fmt::Display, ops::Deref};

use headers::{Header, HeaderName, HeaderValue};
use percent_encoding::{percent_decode, utf8_percent_encode, AsciiSet, CONTROLS};

/// `Slug` header is defined in [`rfc5023`](https://datatracker.ietf.org/doc/html/rfc5023#section-9.7)
///
/// The syntax of the Slug header is defined using the augmented BNF
/// syntax defined in Section 2.1 of RFC2616:
///
/// ```txt
///     LWS      = <defined in Section 2.2 of [RFC2616]>
///     slugtext = %x20-7E | LWS
///     Slug     = "Slug" ":" *slugtext
///```
/// The field value is the percent-encoded value of the UTF-8 encoding of
/// the character sequence to be included (see Section 2.1 of RFC3986
/// for the definition of percent encoding, and RFC3629 for the
/// definition of the UTF-8 encoding).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slug {
    /// Pct decoded slug text.
    pct_decoded_slugtext: String,
}

/// Static for `slug` header-name.
pub static SLUG: HeaderName = HeaderName::from_static("slug");

/// Static for ascii-set to be encoded in slug header.
pub static SLUG_ENCODE_ASCII_SET: AsciiSet = CONTROLS.add(b'%');

impl Header for Slug {
    fn name() -> &'static HeaderName {
        &SLUG
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let mut slugtext = String::new();
        for value in values {
            // pct decode bytes first, and then decode utf8 str, as per spec
            let pct_decoded_value = percent_decode(value.as_bytes()).decode_utf8_lossy();

            if !slugtext.is_empty() {
                // see <https://stackoverflow.com/a/38406581>
                slugtext.push(',');
            }
            slugtext.push_str(pct_decoded_value.as_ref());
        }
        Ok(Self {
            pct_decoded_slugtext: slugtext,
        })
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            HeaderValue::from_str(
                Into::<Cow<str>>::into(utf8_percent_encode(
                    &self.pct_decoded_slugtext,
                    &SLUG_ENCODE_ASCII_SET,
                ))
                .as_ref(),
            )
            .expect("Must be valid header"),
        ))
    }
}

impl Display for Slug {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pct_decoded_slugtext.fmt(f)
    }
}

impl Deref for Slug {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.pct_decoded_slugtext
    }
}

impl AsRef<str> for Slug {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.pct_decoded_slugtext
    }
}

impl From<String> for Slug {
    #[inline]
    fn from(s: String) -> Self {
        Self {
            pct_decoded_slugtext: s,
        }
    }
}

impl From<&str> for Slug {
    #[inline]
    fn from(s: &str) -> Self {
        Self {
            pct_decoded_slugtext: s.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use claims::*;
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(&[""], "")]
    #[case(&["a b"], "a b")]
    #[case(&["abc", "def"], "abc,def")]
    #[case(&["%E0%A4%B0%E0%A4%BE%E0%A4%AE %E0%A4%B2%E0%A4%95%E0%A5%8D%E0%A4%B7%E0%A5%8D%E0%A4%AE%E0%A4%A3"], "राम लक्ष्मण")]
    fn decode_works_correctly(#[case] header_value_strs: &[&str], #[case] expected_slug_str: &str) {
        let header_values: Vec<HeaderValue> = header_value_strs
            .iter()
            .map(|v| assert_ok!(HeaderValue::from_str(v)))
            .collect();
        let slug = assert_ok!(Slug::decode(&mut header_values.iter()));
        assert_eq!(slug.as_ref(), expected_slug_str);
    }

    #[rstest]
    #[case("a b", "a b")]
    #[case("a/b", "a/b")]
    #[case("a%b", "a%25b")]
    #[case("राम लक्ष्मण", "%E0%A4%B0%E0%A4%BE%E0%A4%AE %E0%A4%B2%E0%A4%95%E0%A5%8D%E0%A4%B7%E0%A5%8D%E0%A4%AE%E0%A4%A3")]
    fn encode_works_correctly(#[case] slug_str: &str, #[case] expected_header_str: &str) {
        let slug: Slug = slug_str.to_string().into();
        let mut headers = Vec::<HeaderValue>::new();
        slug.encode(&mut headers);

        let encoded_header = headers.first().expect("Slug value not encoded");
        let encoded_header_str = assert_ok!(encoded_header.to_str(), "Encoding corruption");
        assert_eq!(encoded_header_str, expected_header_str, "Invalid encoding");
    }
}
