//! I define typed header for `Range` header.
//!

use std::{
    fmt::{Display, Write},
    ops::Deref,
    str::FromStr,
};

use ecow::{eco_format, EcoString};
use headers::Header;
use http::{header::RANGE, HeaderName, HeaderValue};
use http_range_header::parse_range_header;
pub use http_range_header::{ParsedRanges, RangeUnsatisfiableError};
use tracing::error;

/// Typed header for `Range`., that only allows `bytes` range.`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BytesRange {
    raw: EcoString,
    parsed: ParsedRanges,
}


/// Typed header for `Range`., Currently only allows `bytes` range.
pub type Range = BytesRange;

impl Deref for BytesRange {
    type Target = ParsedRanges;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.parsed
    }
}

impl Header for BytesRange {
    #[inline]
    fn name() -> &'static HeaderName {
        &RANGE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let ranges_spec_str = values
            .next()
            .ok_or_else(headers::Error::invalid)?
            .to_str()
            .map_err(|_| headers::Error::invalid())?;

        let parsed_ranges = parse_range_header(ranges_spec_str).map_err(|e| {
            error!("Invalid ranges specifier. {e}");
            headers::Error::invalid()
        })?;

        Ok(Self {
            raw: EcoString::from(ranges_spec_str),
            parsed: parsed_ranges,
        })
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            HeaderValue::from_str(&self.raw).expect("Must be valid header value."),
        ))
    }
}

impl FromStr for BytesRange {
    type Err = RangeUnsatisfiableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            raw: s.into(),
            parsed: parse_range_header(s)?,
        })
    }
}

impl Display for BytesRange {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.raw.fmt(f)
    }
}

impl BytesRange {
    /// Get the string representation.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Create a new [`BytesRange`] with given range set.
    pub fn new(first_range: &BytesRangeValueSpec, rest: &[BytesRangeValueSpec]) -> Self {
        let mut raw = eco_format!("bytes={first_range}");
        rest.iter()
            .for_each(|s| write!(raw, ",{s}").expect("Must be ok"));

        Self {
            parsed: parse_range_header(&raw).expect("Must be valid."),
            raw,
        }
    }
}

/// A struct representing value of a single bytes range.
/// It can be in normal or suffix mode.
/// The byte positions specified are inclusive.  Byte offsets start at zero
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytesRangeValueSpec {
    /// Normal range value.
    Normal {
        /// byte-offset of the first byte in a range.
        first_byte_pos: u64,

        /// byte-offset of the last byte in the range.
        last_byte_pos: Option<u64>,
    },

    /// > A client can request the last N bytes of the
    /// > selected representation using a suffix-byte-range-spec.
    Suffix {
        /// Suffix length.
        length: u64,
    },
}

impl Display for BytesRangeValueSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // > byte-range-spec = first-byte-pos "-" [ last-byte-pos ]
            Self::Normal {
                first_byte_pos,
                last_byte_pos,
            } => {
                write!(f, "{first_byte_pos}-")?;
                if let Some(v) = last_byte_pos {
                    write!(f, "{v}")?;
                }
                Ok(())
            }

            // suffix-byte-range-spec = "-" suffix-length
            Self::Suffix { length } => write!(f, "-{length}"),
        }
    }
}

impl From<BytesRangeValueSpec> for BytesRange {
    #[inline]
    fn from(value: BytesRangeValueSpec) -> Self {
        Self::new(&value, &[])
    }
}

#[cfg(test)]
mod tests {

    use rstest::*;

    use super::{BytesRange, BytesRangeValueSpec};

    #[rstest]
    #[case(BytesRangeValueSpec::Normal{
        first_byte_pos: 100,
        last_byte_pos: Some(199)
    }, "100-199")]
    #[case(BytesRangeValueSpec::Normal{
        first_byte_pos: 200,
        last_byte_pos: None
    }, "200-")]
    #[case(BytesRangeValueSpec::Suffix { length: 100 }, "-100")]

    fn value_spec_display_works_correctly(
        #[case] spec: BytesRangeValueSpec,
        #[case] expected: &str,
    ) {
        assert_eq!(spec.to_string(), expected);
    }

    #[rstest]
    #[case(BytesRangeValueSpec::Normal{
        first_byte_pos: 100,
        last_byte_pos: Some(199)
    }, &[], "bytes=100-199")]
    #[case(BytesRangeValueSpec::Normal{
        first_byte_pos: 100,
        last_byte_pos: Some(199)
    }, &[
        BytesRangeValueSpec::Normal{
        first_byte_pos: 200,
        last_byte_pos: None
    },
    BytesRangeValueSpec::Suffix { length: 600 }
    ], "bytes=100-199,200-,-600")]
    fn new_range_works_correctly(
        #[case] first_range: BytesRangeValueSpec,
        #[case] rest: &[BytesRangeValueSpec],
        #[case] expected: &str,
    ) {
        let range = BytesRange::new(&first_range, rest);
        assert_eq!(range.as_str(), expected);
    }
}
