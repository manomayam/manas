//! I define rust model for `access-mode` production.
//!

use std::{fmt::Display, ops::Deref, str::FromStr};

use ecow::EcoString;
use unicase::Ascii;

static ALL_MODES: [Ascii<&str>; 4] = [
    Ascii::new("read"),
    Ascii::new("write"),
    Ascii::new("append"),
    Ascii::new("control"),
];

/// A struct for representing `access-mode`  abnf production.
/// from [WAC specification](https://solid.github.io/web-access-control-spec/#wac-allow).
///
/// > access-mode      = "read" / "write" / "append" / "control"
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccessMode(Ascii<EcoString>);

impl AccessMode {
    /// `read` access mode.
    pub const READ: Self = Self(Ascii::new(EcoString::inline("read")));

    /// `write` access mode.
    pub const WRITE: Self = Self(Ascii::new(EcoString::inline("write")));

    /// `append` access mode.
    pub const APPEND: Self = Self(Ascii::new(EcoString::inline("append")));

    /// `control` access mode.
    pub const CONTROL: Self = Self(Ascii::new(EcoString::inline("control")));
}

impl Display for AccessMode {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Invalid access mode.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Invalid access mode.")]
pub struct InvalidAccessMode {}

impl FromStr for AccessMode {
    type Err = InvalidAccessMode;

    #[inline]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if ALL_MODES.contains(&Ascii::new(value)) {
            Ok(Self(Ascii::new(value.into())))
        } else {
            Err(InvalidAccessMode {})
        }
    }
}

impl Deref for AccessMode {
    type Target = Ascii<EcoString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("abcd")]
    #[case("a")]
    fn invalid_str_will_be_rejected(#[case] token_str: &str) {
        assert_err_eq!(AccessMode::from_str(token_str), InvalidAccessMode {});
    }

    #[rstest]
    #[case("read")]
    #[case("write")]
    #[case("append")]
    #[case("control")]
    fn valid_str_will_be_parsed(#[case] token_str: &str) {
        assert_ok!(AccessMode::from_str(token_str));
    }
}
