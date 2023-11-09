//! I define type for representing `challenge` production as defined in rfc9110.

use std::str::FromStr;

use either::Either;
use unicase::Ascii;

use super::token68::Token68;
use crate::common::field::rules::{
    flat_csv::{split_field_params, Comma},
    parameters::FieldParameters,
    token::Token,
};

/// A struct for representing `challenge` production as defined in rfc9110.
///
/// ```txt
///     auth-scheme    = token
///     token68        = 1*( ALPHA / DIGIT /
///                                 "-" / "." / "_" / "~" / "+" / "/" ) *"="
///     auth-param     = token BWS "=" BWS ( token / quoted-string )
///     challenge   = auth-scheme [ 1*SP ( token68 / #auth-param ) ]
///```
#[derive(Debug, Clone)]
pub struct Challenge {
    /// Auth scheme of the challenge.
    pub auth_scheme: Ascii<Token>,

    /// Additional info of the challenge.
    pub ext_info: Either<Token68, FieldParameters<Comma>>,
}

impl FromStr for Challenge {
    type Err = InvalidEncodedChallenge;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (auth_scheme_str, info_str) =
            s.trim().split_once(' ').ok_or(InvalidEncodedChallenge)?;

        let auth_scheme =
            Ascii::new(Token::from_str(auth_scheme_str).map_err(|_| InvalidEncodedChallenge)?);

        let ext_info = if let Ok(t) = Token68::from_str(info_str) {
            Either::Left(t)
        } else {
            Either::Right(
                FieldParameters::<Comma>::decode(split_field_params::<Comma>(info_str), false)
                    .map_err(|_| InvalidEncodedChallenge)?,
            )
        };

        Ok(Self {
            auth_scheme,
            ext_info,
        })
    }
}

impl Challenge {
    /// Encode challenge as http field.
    pub fn encode(&self) -> String {
        let mut buf = String::new();

        buf.push_str(self.auth_scheme.as_ref());

        buf.push(' ');

        match &self.ext_info {
            Either::Left(t) => buf.push_str(t.as_str()),
            Either::Right(ps) => ps.push_encoded_str(&mut buf),
        }

        buf
    }
}

/// An error type for invalid encoded challenge.
#[derive(Debug, thiserror::Error)]
#[error("Invalid encoded challenge.")]
pub struct InvalidEncodedChallenge;

// TODO tests
