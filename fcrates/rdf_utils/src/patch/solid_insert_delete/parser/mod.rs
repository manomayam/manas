#![deny(unsafe_code)]

mod error;
mod shared;
mod triple_allocator;
mod turtle;
mod utils;

mod n3_simple;

// pub use error::TurtleError;
pub use n3_simple::N3SimpleParser;

/// Maximal number of nested structures (collections, blank node, quoted triples...).
///
/// This limit is set in order to avoid stack overflow error when parsing such structures due to too many recursive calls.
/// The actual limit value is a wet finger compromise between not failing to parse valid files and avoiding to trigger stack overflow errors.
const MAX_STACK_SIZE: usize = 128;
