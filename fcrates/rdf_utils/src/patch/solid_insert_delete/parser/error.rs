use std::{char, error::Error, fmt, io};

use oxilangtag::LanguageTagParseError;
use oxiri::IriParseError;
use rio_api::parser::{LineBytePosition, ParseError};

use super::MAX_STACK_SIZE;

/// Error that might be returned during parsing.
///
/// It might wrap an IO error or be a parsing error.
#[derive(Debug)]
pub struct TurtleError {
    pub(crate) kind: TurtleErrorKind,
    pub(crate) position: Option<LineBytePosition>,
}

#[derive(Debug)]
pub enum TurtleErrorKind {
    Io(io::Error),
    UnknownPrefix(String),
    PrematureEof,
    UnexpectedByte(u8),
    InvalidUnicodeCodePoint(u32),
    InvalidIri {
        iri: String,
        error: IriParseError,
    },
    InvalidLanguageTag {
        tag: String,
        error: LanguageTagParseError,
    },
    StackOverflow,
}

impl fmt::Display for TurtleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TurtleErrorKind::Io(error) => return error.fmt(f),
            TurtleErrorKind::UnknownPrefix(prefix) => write!(f, "unknown prefix '{}'", prefix),
            TurtleErrorKind::PrematureEof => write!(f, "premature end of file"),
            TurtleErrorKind::UnexpectedByte(c) => match char::from_u32(u32::from(*c)) {
                Some(c) => write!(f, "unexpected character '{}'", c.escape_debug()),
                None => write!(f, "unexpected byter {}", c),
            },
            TurtleErrorKind::InvalidUnicodeCodePoint(point) => {
                write!(f, "invalid unicode code point '{}'", point)
            }
            TurtleErrorKind::InvalidIri { iri, error } => {
                write!(f, "error while parsing IRI '{}': {}", iri, error)
            }
            TurtleErrorKind::InvalidLanguageTag { tag, error } => {
                write!(f, "error while parsing language tag '{}': {}", tag, error)
            }
            TurtleErrorKind::StackOverflow => {
                write!(f, "The parser encountered more than {} nested constructions. This number is limited in order to avoid stack overflow OS errors.", MAX_STACK_SIZE)
            }
        }?;
        if let Some(position) = self.position {
            write!(
                f,
                " on line {} at position {}",
                position.line_number(),
                position.byte_number(),
            )?;
        }
        Ok(())
    }
}

impl Error for TurtleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            TurtleErrorKind::Io(error) => Some(error),
            TurtleErrorKind::InvalidIri { error, .. } => Some(error),
            TurtleErrorKind::InvalidLanguageTag { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl ParseError for TurtleError {
    fn textual_position(&self) -> Option<LineBytePosition> {
        self.position
    }
}

impl From<io::Error> for TurtleError {
    fn from(error: io::Error) -> Self {
        Self {
            kind: TurtleErrorKind::Io(error),
            position: None,
        }
    }
}

impl From<TurtleError> for io::Error {
    fn from(error: TurtleError) -> Self {
        match error.kind {
            TurtleErrorKind::Io(error) => error,
            TurtleErrorKind::PrematureEof => io::Error::new(io::ErrorKind::UnexpectedEof, error),
            _ => io::Error::new(io::ErrorKind::InvalidData, error),
        }
    }
}
