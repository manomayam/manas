//! This modules provides utils for quads serialization.
//!

mod factory;
mod sync;

#[cfg(feature = "async")]
mod async_;

#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
#[cfg(feature = "async")]
pub use async_::DynSynAsyncQuadSerializer;
pub use factory::DynSynQuadSerializerFactory;
pub use sync::DynSynQuadSerializer;

#[cfg(feature = "async")]
pub(crate) type BridgedWrite<W> =
    tokio_util::io::SyncIoBridge<tokio::io::BufWriter<async_compat::Compat<W>>>;
