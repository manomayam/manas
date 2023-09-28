//! I define [`QValue`] struct that is used for representing
//! `q` parameter or `weight`.
use std::{ops::Deref, str::FromStr};

use once_cell::sync::Lazy;
use rust_decimal::Decimal;

use crate::field::rules::parameter_name::FieldParameterName;

/// Constant for "q" field param name.
pub static Q_PARAM_NAME: Lazy<FieldParameterName> =
    Lazy::new(|| "q".parse().expect("Must be valid"));

/// `QValue` value, defined in
/// [RFC7231](https://datatracker.ietf.org/doc/html/rfc7231#section-5.3.1)
/// Many of the request header fields for proactive negotiation use a
/// common parameter, named "q" (case-insensitive), to assign a relative
/// "weight" to the preference for that associated kind of content.  This
/// weight is referred to as a "quality value" (or "qvalue") because the
/// same parameter name is often used within server configurations to
/// assign a weight to the relative quality of the various
/// representations that can be selected for a resource.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QValue(Decimal);

impl Default for QValue {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Error of invalid qvalue.
pub enum InvalidQValue {
    /// Source is not a number.
    #[error("Given source is not a number")]
    SourceIsNotANumber,

    /// Value is out of range.
    #[error("Given value is out of range. qvalue must be in range [0, 1]")]
    ValueOutOfRange,

    /// Value is out of scale.
    #[error(
        "Given value has too many significant digits after decimal. Only up to three are allowed"
    )]
    ValueOutOfScale,
}

impl TryFrom<Decimal> for QValue {
    type Error = InvalidQValue;

    #[inline]
    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        // Ensure value in range [0, 1]
        if value < Decimal::ZERO || value > Decimal::ONE {
            Err(InvalidQValue::ValueOutOfRange)
        }
        // Ensure value scale is in range [0, 3]
        else if value.scale() > 3 {
            Err(InvalidQValue::ValueOutOfScale)
        } else {
            Ok(Self(value))
        }
    }
}

impl FromStr for QValue {
    type Err = InvalidQValue;

    #[inline]
    fn from_str(value_str: &str) -> Result<Self, Self::Err> {
        Decimal::from_str_exact(value_str)
            .map_err(|_| InvalidQValue::SourceIsNotANumber)?
            .try_into()
    }
}

impl Deref for QValue {
    type Target = Decimal;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl QValue {
    /// Default QValue as specified by spec.
    pub const DEFAULT: Self = Self(Decimal::ONE);

    /// QValue of 1.
    pub const ONE: Self = Self(Decimal::ONE);

    /// QValue of 0.
    pub const ZERO: Self = Self(Decimal::ZERO);
}

#[cfg(test)]
mod tests_try_from {
    use claims::{assert_err_eq, assert_ok};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("0.a")]
    #[case("abc")]
    #[case("0 1")]
    fn non_number_string_will_be_rejected(#[case] value_str: &str) {
        assert_err_eq!(
            QValue::from_str(value_str),
            InvalidQValue::SourceIsNotANumber
        );
    }

    #[rstest]
    #[case(Decimal::new(23, 1))]
    #[case(Decimal::new(-19, 1))]
    #[case(Decimal::new(-1, 0))]
    fn number_out_of_range_will_be_rejected(#[case] value: Decimal) {
        assert_err_eq!(QValue::try_from(value), InvalidQValue::ValueOutOfRange);
    }

    #[rstest]
    #[case(Decimal::new(2356, 4))]
    #[case(Decimal::new(123, 4))]
    #[case(Decimal::new(5, 4))]
    #[case(Decimal::from_str_exact("0.1456").unwrap())]
    #[case(Decimal::from_str_exact("0.0001").unwrap())]
    #[case(Decimal::from_str_exact("0.9999").unwrap())]
    #[case(Decimal::from_str_exact("0.0001").unwrap())]
    #[case(Decimal::from_str_exact("0.9999").unwrap())]
    fn number_out_of_scale_will_be_rejected(#[case] value: Decimal) {
        assert_err_eq!(QValue::try_from(value), InvalidQValue::ValueOutOfScale);
    }

    #[rstest]
    #[case("0")]
    #[case("1")]
    #[case("0.001")]
    #[case("0.1")]
    #[case("0.9")]
    #[case("0.999")]
    #[case("0.56")]
    #[case("1.0")]
    fn valid_str_will_be_accepted(#[case] value_str: &str) {
        assert_ok!(QValue::from_str(value_str));
    }

    #[rstest]
    #[case(Decimal::new(123, 3))]
    #[case(Decimal::new(1, 0))]
    #[case(Decimal::new(0, 1))]
    #[case(Decimal::new(12, 2))]
    #[case(Decimal::new(1, 3))]
    #[case(Decimal::new(123, 3))]
    fn valid_decimal_number_will_be_accepted(#[case] value: Decimal) {
        assert_ok!(QValue::try_from(value));
    }
}
