//! This modules provides utils for parsing quads.
//!

mod factory;
mod source;
mod sync;

#[cfg(feature = "async")]
mod async_;

#[cfg(feature = "async")]
pub use async_::DynSynAsyncTripleParser;
pub use factory::DynSynTripleParserFactory;
pub use source::DynSynTripleSource;
pub use sync::DynSynTripleParser;
