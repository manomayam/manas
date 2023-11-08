//! This crate provides types for representing http uris and their invariants.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::{
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

use iri_string::{
    components::AuthorityComponents,
    format::ToDedicatedString,
    types::{UriReferenceStr, UriStr, UriString},
};
use unicase::Ascii;

#[cfg(feature = "invariants")]
pub mod invariant;
#[cfg(feature = "invariants")]
pub mod predicate;

pub mod security;

/// Http scheme.
pub const HTTP_SCHEME: Ascii<&'static str> = Ascii::new("http");
/// Https scheme.
pub const HTTPS_SCHEME: Ascii<&'static str> = Ascii::new("https");

/// Default port for http scheme.
pub const HTTP_DEFAULT_PORT: &str = "80";
/// Default port for https scheme.
pub const HTTPS_DEFAULT_PORT: &str = "443";

/// A O(1) clonable struct for representing http uri.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct HttpUri {
    uri: Arc<UriStr>,
}

impl Debug for HttpUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpUri({})", self.uri.as_str())
    }
}

impl Display for HttpUri {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.uri.as_str(), f)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for HttpUri {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.uri.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for HttpUri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Expected;

        impl serde::de::Expected for Expected {
            fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "valid http uri")
            }
        }

        let uri = UriString::deserialize(deserializer)?;

        Self::try_from(uri.as_slice()).map_err(|_| {
            <D::Error as serde::de::Error>::invalid_value(
                serde::de::Unexpected::Other("invalid http uri"),
                &Expected,
            )
        })
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid http uri.
pub enum InvalidHttpUri {
    /// Invalid uri.
    #[error("Given source is not a valid uri")]
    InvalidUri,

    /// Empty host in uri.
    #[error("Given source uri has empty host")]
    EmptyHost,

    /// Non http scheme in uri.
    #[error("Given source uri has non http scheme")]
    NonHttpScheme,
}

impl TryFrom<&UriStr> for HttpUri {
    type Error = InvalidHttpUri;

    fn try_from(uri: &UriStr) -> Result<Self, Self::Error> {
        // Ensure scheme is http/https
        let scheme = Ascii::new(uri.scheme_str());
        if scheme != HTTP_SCHEME && scheme != HTTPS_SCHEME {
            return Err(InvalidHttpUri::NonHttpScheme);
        }

        // Ensure host is non-empty.
        // > A sender MUST NOT generate an "http(s)" URI with an empty host identifier.
        // > A recipient that processes such a URI reference MUST reject it as invalid.
        let is_empty_host = uri
            .authority_components()
            .map(|a| a.host().is_empty())
            .unwrap_or(true);
        if is_empty_host {
            return Err(InvalidHttpUri::EmptyHost);
        }
        Ok(Self {
            uri: Arc::from(uri),
        })
    }
}

impl TryFrom<&str> for HttpUri {
    type Error = InvalidHttpUri;

    #[inline]
    fn try_from(uri_str: &str) -> Result<Self, Self::Error> {
        let uri: &UriStr = uri_str.try_into().map_err(|_| InvalidHttpUri::InvalidUri)?;
        uri.try_into()
    }
}

impl FromStr for HttpUri {
    type Err = InvalidHttpUri;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl Deref for HttpUri {
    type Target = UriStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.uri.as_ref()
    }
}

impl AsRef<UriStr> for HttpUri {
    #[inline]
    fn as_ref(&self) -> &UriStr {
        self.uri.as_ref()
    }
}

impl AsRef<UriReferenceStr> for HttpUri {
    #[inline]
    fn as_ref(&self) -> &UriReferenceStr {
        self.uri.as_ref().as_ref()
    }
}

impl AsRef<str> for HttpUri {
    #[inline]
    fn as_ref(&self) -> &str {
        self.uri.as_str()
    }
}

impl From<HttpUri> for UriString {
    #[inline]
    fn from(val: HttpUri) -> Self {
        val.uri.deref().to_owned()
    }
}

impl HttpUri {
    /// Get authority components of http uri
    #[inline]
    pub fn authority_components(&self) -> AuthorityComponents {
        self.uri.authority_components().expect("Checked to be some")
    }

    /// Get authority str of this http uri
    #[inline]
    pub fn authority_str(&self) -> &str {
        self.uri.authority_str().expect("Checked to be some")
    }

    /// Checks if uri is std normalized as per rfc3986
    #[inline]
    pub fn is_rfc3986_normalized(&self) -> bool {
        self.uri.is_normalized_rfc3986()
    }

    /// Returns a std normal http uri. standard normalization follows rfc 3986
    #[inline]
    pub fn normalize_rfc3986(&self) -> HttpUri {
        HttpUri {
            uri: Arc::from(self.uri.normalize().to_dedicated_string().as_ref()),
        }
    }

    /// Check if http uri has explicit http default port.
    fn has_explicit_default_port(&self) -> bool {
        self.authority_components()
            .port()
            .map(|port| {
                let scheme = Ascii::new(self.scheme_str());
                // If port string is empty, but exists with delim.
                port.is_empty()
            // If scheme is http, and port is 80.
                || (scheme == HTTP_SCHEME && port == HTTP_DEFAULT_PORT)
            // If scheme is https, and port is 443.
                || (scheme == HTTPS_SCHEME && port == HTTPS_DEFAULT_PORT)
            })
            .unwrap_or(false)
    }

    /// Checks if uri is http normalized.
    #[inline]
    pub fn is_http_normalized(&self) -> bool {
        // If is normalized as per rfc3986.
        self.is_rfc3986_normalized()
            // Has no explicit default port.
            && !self.has_explicit_default_port()
            // Has non empty path str.
            && !self.uri.path_str().is_empty()
            // Has no non trailing empty segments.
            && !self.uri.path_str().contains("//")
    }

    /// Returns a normal http uri.
    /// This function performs additional http specific normalization along with uri normalization specified by rfc3986.
    ///
    /// Http normalization entails:
    /// - Normalization of port, If default port is explicitly specified,it will be removed.
    /// - Normalization of path: Empty path will be normalized to `/`.
    /// - If there are non-trailing empty segments in path, they will be removed.
    ///
    pub fn http_normalized(&self) -> HttpUri {
        // Perform uri normalization as per rfc3986
        let rfc3986_normalized = self.normalize_rfc3986();

        // Check if uri has explicit default port.
        let has_explicit_default_port = rfc3986_normalized.has_explicit_default_port();

        // Check if uri has empty path.
        let has_empty_path = rfc3986_normalized.path_str().is_empty();

        // Check if uri has non trailing empty segment..
        let has_non_trailing_empty_segment = rfc3986_normalized.path_str().contains("//");

        // If http normalized, return it.
        if !has_explicit_default_port && !has_empty_path && !has_non_trailing_empty_segment {
            return rfc3986_normalized;
        }

        let mut buffer = String::with_capacity(rfc3986_normalized.len());
        // Push scheme
        buffer.push_str(rfc3986_normalized.scheme_str());
        // Push scheme-separator.
        buffer.push_str("://");
        // Push authority
        buffer.push_str(if !has_explicit_default_port {
            rfc3986_normalized.authority_str()
        } else {
            rfc3986_normalized
                .authority_str()
                .rsplit_once(':')
                .expect("Must be some, as port is some")
                .0
        });

        // Push normalized path
        let mut is_prev_slash = false;
        for ch in rfc3986_normalized.path_str().chars() {
            let is_slash = ch == '/';
            if !(is_slash && is_prev_slash) {
                buffer.push(ch);
            }
            is_prev_slash = is_slash;
        }

        // Normalize non empty path..
        if has_empty_path {
            buffer.push('/');
        }

        // Push query
        if let Some(query) = rfc3986_normalized.query_str() {
            buffer.push('?');
            buffer.push_str(query);
        }

        // Push fragment.
        if let Some(fragment) = rfc3986_normalized.fragment() {
            buffer.push('#');
            buffer.push_str(fragment.as_str());
        }

        buffer.as_str().try_into().expect("Must be valid")
    }

    /// Get if uri is https.
    #[inline]
    pub fn is_https(&self) -> bool {
        HTTPS_SCHEME.eq_ignore_ascii_case(self.scheme_str())
    }

    /// Get if http uri's host is localhost.
    pub fn is_localhost(&self) -> bool {
        let host = self.authority_components().host();
        // Localhost or subdomains.
        host == "localhost" || host.ends_with(".localhost")
    }

    /// Get uri as string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.uri.as_str()
    }

    /// Get inner uri.
    #[inline]
    pub fn inner(&self) -> &UriStr {
        self.uri.as_ref()
    }
}

#[cfg(feature = "sophia")]
impl sophia_api::term::Term for HttpUri {
    type BorrowTerm<'x> = &'x Self;

    #[inline]
    fn kind(&self) -> sophia_api::term::TermKind {
        sophia_api::term::TermKind::Iri
    }

    #[inline]
    fn borrow_term(&self) -> Self::BorrowTerm<'_> {
        self
    }

    #[inline]
    fn iri(&self) -> Option<sophia_api::term::IriRef<sophia_api::MownStr>> {
        Some(sophia_api::term::IriRef::new_unchecked(
            sophia_api::MownStr::from_str(self.as_str()),
        ))
    }
}

#[cfg(feature = "sophia")]
impl From<&HttpUri> for sophia_api::prelude::Iri<String> {
    fn from(value: &HttpUri) -> Self {
        sophia_api::prelude::Iri::new_unchecked(value.uri.as_str().to_owned())
    }
}

#[cfg(feature = "sophia")]
impl From<&HttpUri> for sophia_api::prelude::Iri<Arc<str>> {
    fn from(value: &HttpUri) -> Self {
        sophia_api::prelude::Iri::new_unchecked(Arc::from(value.uri.as_str()))
    }
}

#[cfg(test)]
mod tests_convert {
    use claims::{assert_err_eq, assert_ok};
    use rstest::*;

    use super::*;

    #[rstest]
    #[case::invalid_char1("http://pod1.example.org/path/to/a b")]
    #[case::invalid_char2("http://pod1.example.org/a\nb")]
    #[case::invalid_char3("http://pod1.example.org/?a\\b")]
    #[case::invalid_char3("http://pod1.example.org/a%b")]
    #[case::invalid_char4("http://pod1.example.org/rama<>sita/doc")]
    #[case::invalid_non_ascii1("http://pod1.example.org/राम")]
    #[case::invalid_non_ascii2("http://pod1.example.org/అయోధ్య")]
    #[case::invalid_gen_delim3("http://pod1.example.org/a/b[c")]
    #[case::invalid_gen_delim4("http://pod1.example.org/a/b]c")]
    fn invalid_uri_will_be_rejected(#[case] uri_str: &str) {
        assert_err_eq!(HttpUri::try_from(uri_str), InvalidHttpUri::InvalidUri);
    }

    #[rstest]
    #[case::ftp("ftp://pod1.example.org/path/to/a")]
    #[case::urn("urn:pod1.example.org::ab")]
    #[case::data("data://pod1.example.org/?ab")]
    #[case::file("file://pod1.example.org/ab")]
    fn uri_with_non_http_scheme_will_be_rejected(#[case] uri_str: &'static str) {
        assert_err_eq!(HttpUri::try_from(uri_str), InvalidHttpUri::NonHttpScheme);
    }

    #[rstest]
    #[case::http_no_authority("http:/path/to/a")]
    #[case::https_no_authority("https:/ab")]
    #[case::http_empty_host("http:///a/b?q")]
    #[case::https_empty_host("https:///a/b?q")]
    fn uri_with_invalid_host_will_be_rejected(#[case] uri_str: &'static str) {
        assert_err_eq!(HttpUri::try_from(uri_str), InvalidHttpUri::EmptyHost);
    }

    #[rstest]
    #[case::origin_only("http://pod1.example.org")]
    #[case::explicit_root_path("http://pod1.example.org/")]
    #[case::un_normalized_scheme("HTTP://pod1.example.org/")]
    #[case::un_normalized_authority("http://pod1.EXAMPLE.org/")]
    #[case::ip_authority("http://127.0.0.1/")]
    #[case::localhost("http://localhost/")]
    #[case::explicit_default_port("http://pod1.example.org:80/")]
    #[case::https("https://pod1.example.org/")]
    #[case::explicit_default_port_2("https://pod1.example.org:443/")]
    #[case::with_query("http://pod1.example.org/a/b?q")]
    #[case::un_normalized_path_1("http://pod1.example.org/a//b")]
    #[case::un_normalized_path_2("http://pod1.example.org/a/b%41c")]
    #[case::un_normalized_path_3("http://pod1.example.org/c%2fd")]
    #[case("http://pod1.example.org/a/b")]
    #[case::with_fragment("http://pod1.example.org/a#bc")]
    #[case::with_query_fragment("http://pod1.example.org/a?b#c")]
    fn valid_http_uri_will_be_accepted(#[case] uri_str: &'static str) {
        assert_ok!(HttpUri::try_from(uri_str));
    }
}

#[cfg(test)]
mod tests_normal_check {
    use rstest::*;

    use super::*;

    pub fn check_if_http_normal(uri_str: &str) -> bool {
        HttpUri::try_from(uri_str)
            .expect("Claimed valid")
            .is_http_normalized()
    }

    #[rstest]
    #[case::a("http://pod1.example.org/kosala/%61yodhya")]
    #[case::a_q("http://pod1.example.org/kosala?%61yodhya")]
    #[case::a_cap("http://pod1.example.org/kosala/%41yodhya")]
    #[case::dig_1("http://pod1.example.org/kosala/raghu%31")]
    #[case::hyphen("http://pod1.example.org/rama%2Drajya")]
    #[case::tilde("http://pod1.example.org/rama%7Esita")]
    #[case::tilde_q("http://pod1.example.org/?rama%7Esita")]
    #[case::period("http://pod1.example.org/kosala%2Eayodhya")]
    #[case::period_2("http://pod1.example.org/path/to/%2E%2E/c")]
    #[case::underscore("http://pod1.example.org/rama%5Flakshmana")]
    fn http_uri_un_normal_with_un_reserved_char_encoded_will_be_non_normal(
        #[case] uri_str: &'static str,
    ) {
        assert!(
            !check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }

    #[rstest]
    #[case("http://pod1.example.org/rama%3ddharma")]
    #[case("http://pod1.example.org/?rama%3ddharma")]
    #[case("http://pod1.example.org/rama%2blakshmana")]
    #[case("http://pod1.example.org/rama%2clakshmana")]
    #[case("http://pod1.example.org/%e0%A4%b0%E0%A4%BE%E0%A4%AE")]
    #[case("http://pod1.example.org/the?%e0%A4%b0%E0%A4%BE%E0%A4%AE")]
    #[case("http://pod1.example.org/kosala%2fayodhya")]
    fn http_uri_un_normal_with_lowercase_pct_encoded_will_be_non_normal(
        #[case] uri_str: &'static str,
    ) {
        assert!(
            !check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }

    #[rstest]
    // #[case("HTTP://pod1.example.org/rama")] ; scheme is auto normalized by parser
    #[case("http://pod1.EXAMPLE.org/")]
    #[case("http://pod1.example.OEG/rama%2blakshmana")]
    #[case("http://pod1.ExAmple.org/rama%2clakshmana")]
    fn http_uri_un_normal_with_uppercase_origin_char_will_be_non_normal(
        #[case] uri_str: &'static str,
    ) {
        assert!(
            !check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }

    #[rstest]
    #[case("http://pod1.example.org/.")]
    #[case("http://pod1.example.org/..")]
    #[case("http://pod1.example.org/./path/to/a")]
    #[case("http://pod1.example.org/../a")]
    #[case("http://pod1.example.org/path/to/a/../b/")]
    #[case("http://pod1.example.org/path/to/c/.")]
    #[case("http://pod1.example.org/path/to/c/./")]
    #[case("http://pod1.example.org/path/to/c/..")]
    #[case("http://pod1.example.org/path/to/c/../")]
    fn http_uri_un_normal_with_dot_segments_will_be_non_normal(#[case] uri_str: &'static str) {
        assert!(
            !check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }

    #[rstest]
    #[case::http_80("http://pod1.example.org:80/path/to/a")]
    #[case::https_443("https://pod1.example.org:443/a?b")]
    #[case::http_localhost_80("http://localhost:80/")]
    fn http_uri_with_explicit_default_port_will_be_non_normal(#[case] uri_str: &'static str) {
        assert!(
            !check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }

    #[rstest]
    #[case("http://pod1.example.org/path/to//a")]
    #[case("http://pod1.example.org/path/to//")]
    fn http_uri_with_non_trailing_empty_segment_will_be_non_normal(#[case] uri_str: &'static str) {
        assert!(
            !check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }

    #[rstest]
    #[case("http://pod1.example.org")]
    #[case("http://localhost")]
    fn http_uri_with_empty_path_will_be_non_normal(#[case] uri_str: &'static str) {
        assert!(
            !check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }

    #[rstest]
    #[case::root("http://pod1.example.org/")]
    #[case::localhost_authority("http://localhost/a/b/c")]
    #[case::ip_authority("http://127.0.0.1/a/b/c")]
    #[case::explicit_port1("http://pod1.example.org:8000/a/b/c")]
    #[case::explicit_port2("http://pod1.example.org:8443/")]
    #[case::explicit_port2("http://pod1.example.org:8443/")]
    #[case::non_relative_period("http://pod1.example.org/a.acl")]
    #[case("http://pod1.example.org/kosala/ayodhya")]
    #[case("http://pod1.example.org/path/to/container/")]
    #[case::un_reserved_unencoded_1("http://pod1.example.org/path/to/container/.acl")]
    #[case::un_reserved_unencoded_1_q("http://pod1.example.org/path/to/container/?.acl")]
    #[case::un_reserved_unencoded_2("http://pod1.example.org/path/to/container/~acl")]
    #[case::un_reserved_unencoded_3("http://pod1.example.org/path/to/container/_acl")]
    #[case::un_reserved_unencoded_4("http://pod1.example.org/path/to/container/-acl")]
    #[case::sub_delim_unencoded_1("http://pod1.example.org/path/to/container/$acl")]
    #[case::sub_delim_unencoded_2("http://pod1.example.org/rama=dharma")]
    #[case::sub_delim_unencoded_3("http://pod1.example.org/rama+lakshmana")]
    #[case::sub_delim_unencoded_4("http://pod1.example.org/rama,lakshmana")]
    #[case::sub_delim_unencoded_5("http://pod1.example.org/rama&lakshmana")]
    #[case::sub_delim_encoded_1("http://pod1.example.org/path/to/container/%24acl")]
    #[case::sub_delim_encoded_2("http://pod1.example.org/rama%3Ddharma")]
    #[case::sub_delim_encoded_3("http://pod1.example.org/rama%2Blakshmana")]
    #[case::sub_delim_encoded_4("http://pod1.example.org/rama%2Clakshmana")]
    #[case::sub_delim_encoded_5("http://pod1.example.org/rama%26lakshmana")]
    #[case::excepted_gen_delim_unencoded_1("http://pod1.example.org/a:b")]
    #[case::expected_gen_delim_unencoded_2("http://pod1.example.org/a@b")]
    #[case::excepted_gen_delim_encoded_1("http://pod1.example.org/a%3Ab")]
    #[case::expected_gen_delim_encoded_2("http://pod1.example.org/a%40b")]
    #[case::gen_delim_encoded_1("http://pod1.example.org/a/b%2Fc")]
    #[case::gen_delim_encoded_2("http://pod1.example.org/a/b%3Fc")]
    #[case::gen_delim_encoded_3("http://pod1.example.org/a/b%23c")]
    #[case::gen_delim_encoded_4("http://pod1.example.org/a/b%5B%5Dc")]
    #[case::non_ascii_pct_encoded("http://pod1.example.org/ramayana/%E0%A4%B0%E0%A4%BE%E0%A4%AE")]
    #[case::non_ascii_pct_encoded(
        "http://pod1.example.org/%E0%B0%85%E0%B0%AF%E0%B1%8B%E0%B0%A7%E0%B1%8D%E0%B0%AF"
    )]
    fn normalized_http_uri_will_be_normal(#[case] uri_str: &'static str) {
        assert!(
            check_if_http_normal(uri_str),
            "normalcy of uri \"{}\" is computed incorrectly ",
            uri_str
        );
    }
}

#[cfg(test)]
mod tests_normalization {
    use claims::assert_ok;
    use rstest::*;

    use super::*;

    fn assert_correct_normalization(source_uri_str: &'static str, expected_uri_str: &'static str) {
        let http_uri = assert_ok!(HttpUri::try_from(source_uri_str));
        let normal_http_uri = HttpUri::http_normalized(&http_uri);
        assert_eq!(normal_http_uri.as_str(), expected_uri_str);
    }

    #[rstest]
    #[case::a(
        "http://pod1.example.org/kosala/%61yodhya",
        "http://pod1.example.org/kosala/ayodhya"
    )]
    #[case::a_q(
        "http://pod1.example.org/kosala?%61yodhya",
        "http://pod1.example.org/kosala?ayodhya"
    )]
    #[case::a_cap(
        "http://pod1.example.org/kosala/%41yodhya",
        "http://pod1.example.org/kosala/Ayodhya"
    )]
    #[case::dig_1(
        "http://pod1.example.org/kosala/raghu%31",
        "http://pod1.example.org/kosala/raghu1"
    )]
    #[case::hyphen(
        "http://pod1.example.org/rama%2Drajya",
        "http://pod1.example.org/rama-rajya"
    )]
    #[case::tilde(
        "http://pod1.example.org/rama%7Esita",
        "http://pod1.example.org/rama~sita"
    )]
    #[case::period(
        "http://pod1.example.org/kosala%2Eayodhya",
        "http://pod1.example.org/kosala.ayodhya"
    )]
    #[case::period_2(
        "http://pod1.example.org/path/to/%2E%2E/c",
        "http://pod1.example.org/path/c"
    )]
    #[case::underscore(
        "http://pod1.example.org/rama%5Flakshmana",
        "http://pod1.example.org/rama_lakshmana"
    )]
    fn unreserved_char_encoding_will_be_normalized_correctly(
        #[case] source_uri_str: &'static str,
        #[case] expected_uri_str: &'static str,
    ) {
        assert_correct_normalization(source_uri_str, expected_uri_str);
    }

    #[rstest]
    #[case(
        "http://pod1.example.org/rama%3ddharma",
        "http://pod1.example.org/rama%3Ddharma"
    )]
    #[case(
        "http://pod1.example.org/?rama%3ddharma",
        "http://pod1.example.org/?rama%3Ddharma"
    )]
    #[case(
        "http://pod1.example.org/rama%2blakshmana",
        "http://pod1.example.org/rama%2Blakshmana"
    )]
    #[case(
        "http://pod1.example.org/rama%2clakshmana",
        "http://pod1.example.org/rama%2Clakshmana"
    )]
    #[case(
        "http://pod1.example.org/%e0%A4%b0%E0%A4%BE%E0%A4%AE",
        "http://pod1.example.org/%E0%A4%B0%E0%A4%BE%E0%A4%AE"
    )]
    #[case(
        "http://pod1.example.org/kosala%2fayodhya",
        "http://pod1.example.org/kosala%2Fayodhya"
    )]
    fn pct_encoded_octets_case_will_be_normalized_correctly(
        #[case] source_uri_str: &'static str,
        #[case] expected_uri_str: &'static str,
    ) {
        assert_correct_normalization(source_uri_str, expected_uri_str);
    }

    #[rstest]
    #[case("http://pod1.example.org/.", "http://pod1.example.org/")]
    #[case("http://pod1.example.org/..", "http://pod1.example.org/")]
    #[case(
        "http://pod1.example.org/./path/to/a",
        "http://pod1.example.org/path/to/a"
    )]
    #[case("http://pod1.example.org/../a", "http://pod1.example.org/a")]
    #[case("http://pod1.example.org/./a:b", "http://pod1.example.org/a:b")]
    #[case(
        "http://pod1.example.org/path/to/a/../b/",
        "http://pod1.example.org/path/to/b/"
    )]
    #[case(
        "http://pod1.example.org/path/to/c/.",
        "http://pod1.example.org/path/to/c/"
    )]
    #[case(
        "http://pod1.example.org/path/to/c/./",
        "http://pod1.example.org/path/to/c/"
    )]
    #[case(
        "http://pod1.example.org/path/to/c/..",
        "http://pod1.example.org/path/to/"
    )]
    #[case(
        "http://pod1.example.org/path/to/c/../",
        "http://pod1.example.org/path/to/"
    )]
    #[case::pct_encoded_dot_segments(
        "http://pod1.example.org/path/to/%2E%2E/c",
        "http://pod1.example.org/path/c"
    )]
    fn dot_segments_will_be_normalized_correctly(
        #[case] source_uri_str: &'static str,
        #[case] expected_uri_str: &'static str,
    ) {
        assert_correct_normalization(source_uri_str, expected_uri_str);
    }

    #[rstest]
    #[case::http("http://pod1.example.org/a/b?q", "http://pod1.example.org/a/b?q")]
    #[case::http_80("http://pod1.example.org:80/a/b?q", "http://pod1.example.org/a/b?q")]
    #[case::http_8000(
        "http://pod1.example.org:8000/a/b?q",
        "http://pod1.example.org:8000/a/b?q"
    )]
    #[case::https("https://pod1.example.org/a/b?q", "https://pod1.example.org/a/b?q")]
    #[case::https_443("https://pod1.example.org:443/a/b?q", "https://pod1.example.org/a/b?q")]
    #[case::https_743(
        "https://pod1.example.org:743/a/b?q",
        "https://pod1.example.org:743/a/b?q"
    )]
    fn port_will_be_normalized_correctly(
        #[case] source_uri_str: &'static str,
        #[case] expected_uri_str: &'static str,
    ) {
        assert_correct_normalization(source_uri_str, expected_uri_str);
    }

    #[rstest]
    #[case(
        "http://pod1.example.org/path/to//a",
        "http://pod1.example.org/path/to/a"
    )]
    #[case(
        "http://pod1.example.org/path/to//",
        "http://pod1.example.org/path/to/"
    )]
    fn non_trailing_empty_segments_will_be_normalized_correctly(
        #[case] source_uri_str: &'static str,
        #[case] expected_uri_str: &'static str,
    ) {
        assert_correct_normalization(source_uri_str, expected_uri_str);
    }

    #[rstest]
    #[case("http://pod1.example.org", "http://pod1.example.org/")]
    #[case("http://localhost", "http://localhost/")]
    fn empty_path_will_be_normalized_correctly(
        #[case] source_uri_str: &'static str,
        #[case] expected_uri_str: &'static str,
    ) {
        assert_correct_normalization(source_uri_str, expected_uri_str);
    }
}
