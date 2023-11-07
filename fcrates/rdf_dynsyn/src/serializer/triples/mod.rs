//! This modules provides utils for triples serialization.
//!

mod factory;
mod sync;

#[cfg(feature = "async")]
mod async_;

#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub use async_::DynSynAsyncTripleSerializer;
pub use factory::DynSynTripleSerializerFactory;
pub use sync::DynSynTripleSerializer;
