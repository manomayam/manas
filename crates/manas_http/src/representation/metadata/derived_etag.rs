//! I define types for derived etags.
//!
use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Deref,
    str::FromStr,
};

use headers::{ETag, Header, HeaderValue};
use once_cell::sync::Lazy;
use regex::Regex;
use smallvec::SmallVec;

/// A struct for representing `derived entity tag`.
#[derive(Debug, Clone)]
pub struct DerivedETag {
    /// Derived etag.
    etag: ETag,

    /// ETag str.
    etag_str: String,
}

impl Deref for DerivedETag {
    type Target = ETag;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.etag
    }
}

impl AsRef<ETag> for DerivedETag {
    #[inline]
    fn as_ref(&self) -> &ETag {
        &self.etag
    }
}

impl AsRef<str> for DerivedETag {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.etag_str
    }
}

impl From<ETag> for DerivedETag {
    fn from(etag: ETag) -> Self {
        // Get encoded etag str
        let mut encoded_values = SmallVec::<[HeaderValue; 1]>::new();
        etag.encode(&mut encoded_values);
        let etag_str = encoded_values[0].to_str().expect("Must be valid string");

        Self {
            etag,
            etag_str: etag_str.to_owned(),
        }
    }
}

impl From<DerivedETag> for ETag {
    #[inline]
    fn from(value: DerivedETag) -> Self {
        value.etag
    }
}

impl TryFrom<String> for DerivedETag {
    type Error = <ETag as FromStr>::Err;

    fn try_from(etag_str: String) -> Result<Self, Self::Error> {
        let etag: ETag = etag_str.parse()?;
        Ok(Self { etag, etag_str })
    }
}

impl FromStr for DerivedETag {
    type Err = <ETag as FromStr>::Err;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl DerivedETag {
    /// Get base etag str of this derived etag.
    #[inline]
    pub fn base_etag_str(&self) -> Cow<'_, str> {
        Self::base_normalize_etag_range(&self.etag_str)
    }

    /// Get base etag of this derived etag.
    pub fn base_etag(&self) -> Cow<'_, Self> {
        let base_etag_str = self.base_etag_str();
        // If this detag is base etag itself, then
        if base_etag_str.as_ref() == self.etag_str {
            return Cow::Borrowed(self);
        }

        Cow::Owned(
            Self::try_from(base_etag_str.into_owned())
                .expect("Must be valid etag str, as is substring of other valid etag str."),
        )
    }

    /// Normalize etag range by normalizing each enumerated
    /// etag to it's base rep etag.
    #[inline]
    pub fn base_normalize_etag_range(etag_range_str: &str) -> Cow<'_, str> {
        ETAG_NON_BASE_PART_RE.replace_all(etag_range_str, "")
    }

    /// Get etag for rep generated from current etag's corresponding rep, and with given gen variance key.
    fn generated_rep_etag(&self, variance_key: impl Hash, sep: &str) -> Self {
        // hash variance.
        let variance_hash = ({
            let mut hasher = DefaultHasher::new();
            variance_key.hash(&mut hasher);
            hasher.finish()
        }) / 10000000000;

        // Construct new etag by appending variance hash to encoded opaque value.
        let variant_rep_etag_str = format!(
            "{}{}{}\"",
            self.etag_str
                .strip_suffix('"')
                .expect("Valid encoded etag must contain trailing double quote"),
            sep,
            variance_hash
        );

        Self {
            etag: variant_rep_etag_str.parse().expect("Must be valid etag"),
            etag_str: variant_rep_etag_str,
        }
    }

    /// Get etag for rep derived from current etag's corresponding rep, and with given gen derivation key.
    #[inline]
    pub fn derived_rep_etag(&self, derivation_key: impl Hash) -> Self {
        self.generated_rep_etag(derivation_key, "::")
    }

    /// Get etag for rep augmented from current etag's corresponding rep, and with given gen augmentation key.
    #[inline]
    pub fn augmented_rep_etag(&self, augmentation_key: impl Hash) -> Self {
        self.generated_rep_etag(augmentation_key, "*")
    }
}

static ETAG_NON_BASE_PART_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"::[^"]*"#).expect("Must be valid regex"));

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(r#""""#, "")]
    #[case(r#""xy_zzy""#, "")]
    #[case(r#""xy_zzy""#, "43223")]
    #[case(r#"W/"xy-zzy""#, "abc:deg")]

    fn derived_rep_etag_creation_will_succeed(
        #[case] base_rep_etag_str: &str,
        #[case] derivation_key: &str,
    ) {
        let base_rep_etag: DerivedETag = base_rep_etag_str.parse().expect("Claimed valid");
        let _ = base_rep_etag.derived_rep_etag(derivation_key);
    }

    #[rstest]
    #[case(r#""""#, r#""""#)]
    #[case(r#""abcdef""#, r#""abcdef""#)]
    #[case(r#""a::b""#, r#""a""#)]
    #[case(r#""a::::::b""#, r#""a""#)]
    #[case(r#""kt::kj::kl""#, r#""kt""#)]
    fn base_rep_etag_extraction_works_correctly(
        #[case] derived_rep_etag_str: &str,
        #[case] expected_base_rep_etag_str: &str,
    ) {
        let derived_rep_etag: DerivedETag = derived_rep_etag_str.parse().expect("Claimed valid;");
        let base_rep_etag_str = derived_rep_etag.base_etag_str();

        assert_eq!(base_rep_etag_str.as_ref(), expected_base_rep_etag_str);
        assert_eq!(
            base_rep_etag_str.as_ref(),
            &derived_rep_etag.base_etag().etag_str
        );
    }

    #[rstest]
    #[case("*", "*")]
    #[case(r#""""#, r#""""#)]
    #[case(r#""abcdef", "a::b""#, r#""abcdef", "a""#)]
    #[case(r#""a::b", "::""#, r#""a", """#)]
    #[case(r#""a", "a::::::b", "cde""#, r#""a", "a", "cde""#)]
    #[case(r#""kt::kj::kl""#, r#""kt""#)]
    fn base_normalize_etag_range_works_correctly(
        #[case] etag_range_str: &str,
        #[case] expected_range_str: &str,
    ) {
        assert_eq!(
            DerivedETag::base_normalize_etag_range(etag_range_str).as_ref(),
            expected_range_str
        );
    }
}
