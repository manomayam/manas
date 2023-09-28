//! Copy of <https://github.com/hyperium/headers/blob/master/src/util/flat_csv.rs>

use std::{fmt, marker::PhantomData};

use headers::HeaderValue;

// A single `HeaderValue` that can flatten multiple values with commas.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct FlatCsv<'v, Sep = Comma> {
    pub(crate) value: &'v HeaderValue,
    _marker: PhantomData<Sep>,
}

/// A trait for typed seperator values.
pub trait Separator {
    /// Char of the seperator.
    const CHAR: char;
}

pub(crate) fn split_field_params<Sep: Separator>(
    params_str: &str,
) -> impl Iterator<Item = &'_ str> {
    let mut in_quotes = false;
    params_str
        .split(move |c| {
            if in_quotes {
                if c == '"' {
                    in_quotes = false;
                }
                false // don't split
            } else if c == Sep::CHAR {
                true // split
            } else {
                if c == '"' {
                    in_quotes = true;
                }
                false // don't split
            }
        })
        .map(|item| item.trim())
}

/// An implementation of `Seperator` for `,`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Comma {}

impl Separator for Comma {
    const CHAR: char = ',';
}

/// An implementation of `Seperator` for `.`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SemiColon {}

impl Separator for SemiColon {
    const CHAR: char = ';';
}

impl<'v, Sep: Separator> FlatCsv<'v, Sep> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &'v str> {
        self.value
            .to_str()
            .ok()
            .into_iter()
            .flat_map(|value_str| split_field_params::<Sep>(value_str))
    }
}

impl<'v, Sep> From<&'v HeaderValue> for FlatCsv<'v, Sep> {
    fn from(value: &'v HeaderValue) -> Self {
        FlatCsv {
            value,
            _marker: PhantomData,
        }
    }
}

impl<'v, Sep> fmt::Debug for FlatCsv<'v, Sep> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.value, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comma() {
        let val = HeaderValue::from_static("aaa, b; bb, ccc");
        let csv = FlatCsv::<Comma>::from(&val);

        let mut values = csv.iter();
        assert_eq!(values.next(), Some("aaa"));
        assert_eq!(values.next(), Some("b; bb"));
        assert_eq!(values.next(), Some("ccc"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn semicolon() {
        let val = HeaderValue::from_static("aaa; b, bb; ccc");
        let csv = FlatCsv::<SemiColon>::from(&val);

        let mut values = csv.iter();
        assert_eq!(values.next(), Some("aaa"));
        assert_eq!(values.next(), Some("b, bb"));
        assert_eq!(values.next(), Some("ccc"));
        assert_eq!(values.next(), None);
    }

    #[test]
    fn quoted_text() {
        let val = HeaderValue::from_static("foo=\"bar,baz\", sherlock=holmes");
        let csv = FlatCsv::<Comma>::from(&val);

        let mut values = csv.iter();
        assert_eq!(values.next(), Some("foo=\"bar,baz\""));
        assert_eq!(values.next(), Some("sherlock=holmes"));
        assert_eq!(values.next(), None);
    }
}
