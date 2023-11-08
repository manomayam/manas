//! I define [`MediaType`] struct to represent `media-type` rule.
//!
use std::{fmt::Display, ops::Deref, str::FromStr};

use headers::ContentType;
use mime::Mime;
use once_cell::sync::Lazy;

// `media-type` is defined by [`rfc2616`](https://datatracker.ietf.org/doc/html/rfc2616#section-3.7)
//
/// HTTP uses Internet Media Types in the Content-Type (section
/// 14.17) and Accept (section 14.1) header fields in order to provide
/// open and extensible data typing and type negotiation.
/// ```txt
///     media-type     = type "/" subtype *( ";" parameter )
///     type           = token
///     subtype        = token
/// ```
/// Parameters MAY follow the type/subtype in the form of attribute/value
/// pairs (as defined in section 3.6).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaType(Mime);

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// Error of invalid media type.
pub enum InvalidMediaType {
    /// Invalid media type.
    #[error("Invalid media type")]
    Invalid,
}

impl TryFrom<Mime> for MediaType {
    type Error = InvalidMediaType;

    #[inline]
    fn try_from(value: Mime) -> Result<Self, Self::Error> {
        if value.type_() == mime::STAR || value.subtype() == mime::STAR {
            return Err(InvalidMediaType::Invalid);
        }
        Ok(Self(value))
    }
}

impl TryFrom<ContentType> for MediaType {
    type Error = InvalidMediaType;

    #[inline]
    fn try_from(value: ContentType) -> Result<Self, Self::Error> {
        Self::try_from(Mime::from(value))
    }
}

impl FromStr for MediaType {
    type Err = InvalidMediaType;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let media_range = Mime::from_str(s).map_err(|_| InvalidMediaType::Invalid)?;
        media_range.try_into()
    }
}

impl Display for MediaType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<MediaType> for Mime {
    #[inline]
    fn from(val: MediaType) -> Self {
        val.0
    }
}

impl From<MediaType> for ContentType {
    #[inline]
    fn from(val: MediaType) -> Self {
        val.0.into()
    }
}

impl Deref for MediaType {
    type Target = Mime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MediaType {
    #[inline]
    fn default() -> Self {
        // Defaults to application/octet-stream.
        Self(mime::APPLICATION_OCTET_STREAM)
    }
}

impl MediaType {
    /// Check if media-type is in given media-range.
    pub fn is_in_range(&self, media_range: &Mime) -> bool {
        // If range is "*/*"
        (media_range == &mime::STAR_STAR)
        // If range is "<type>/*", and <type> matches.
            || (media_range.subtype() == mime::STAR && media_range.type_() == self.type_())
        // If essence strings are equal.
            || (media_range.essence_str() == self.essence_str())
    }

    /// Return a &str of the MediaType's "essence".
    #[inline]
    pub fn essence_str(&self) -> &str {
        self.0.essence_str()
    }
}

/// Turtle media type.
pub static TEXT_TURTLE: Lazy<MediaType> =
    Lazy::new(|| "text/turtle".parse().expect("Must be valid"));

/// Turtle media type.
pub static APPLICATION_OCTET_STREAM: Lazy<MediaType> = Lazy::new(|| {
    mime::APPLICATION_OCTET_STREAM
        .try_into()
        .expect("Must be valid")
});

/// json media-type.
pub static APPLICATION_JSON: MediaType = MediaType(mime::APPLICATION_JSON);

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(mime::TEXT_STAR)]
    #[case(mime::IMAGE_STAR)]
    #[case(mime::STAR_STAR)]
    fn invalid_media_range_will_be_rejected(#[case] media_range: Mime) {
        assert_err!(MediaType::try_from(media_range));
    }

    #[rstest]
    #[case(mime::TEXT_HTML_UTF_8)]
    #[case(mime::APPLICATION_WWW_FORM_URLENCODED)]
    #[case(mime::IMAGE_JPEG)]
    #[case(mime::APPLICATION_JAVASCRIPT_UTF_8)]
    fn valid_media_range_will_be_parsed(#[case] media_range: Mime) {
        assert_ok!(MediaType::try_from(media_range));
    }
}
