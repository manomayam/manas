//! I define an inmemory opendal service implementation.
//!

use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use ecow::EcoVec;
use name_locker::impl_::InmemNameLocker;
use opendal::{
    raw::{
        oio, Accessor, AccessorInfo, BytesRange, OpList, OpRead, OpStat, RpList, RpRead, RpStat,
    },
    Capability, Error, ErrorKind, Metadata, Result,
};

use crate::object_store::backend::impl_::common::{
    service_object_path::{NormalPath, NsPath},
    util::apply_range,
};

/// Configuration for [`InmemAccessor`].
#[derive(Debug, Clone)]
pub struct InmemAccessorConfig {
    /// Name of the accessor.
    pub name: String,

    /// Whether to support last modified on objects.
    pub support_last_modified: bool,

    /// Whether to support etags on objects.
    pub support_etag: bool,

    /// Whether to support content-type on objects.
    pub support_cty: bool,

    /// Whether to support independent namespace objects.
    pub support_independent_ns_objects: bool,
}

/// Struct for object entries.
#[derive(Debug)]
struct ObjectState {
    pub content: Bytes,
    pub metadata: Metadata,
}

/// An inmemory opendal service implementation.
#[derive(Debug)]
pub struct InmemAccessor {
    config: InmemAccessorConfig,
    state: DashMap<NormalPath, ObjectState>,
    children_index: DashMap<NsPath, EcoVec<NormalPath>>,
    lock_table: InmemNameLocker<String>,
}

#[async_trait]
impl Accessor for InmemAccessor {
    type Reader = oio::Cursor;
    type BlockingReader = ();
    type Writer = ();
    type BlockingWriter = ();
    type Lister = ();
    type BlockingLister = ();

    fn info(&self) -> AccessorInfo {
        let mut info = AccessorInfo::default();
        info.set_scheme(opendal::Scheme::Custom("Inmem"))
            .set_name(self.config.name.as_str())
            .set_native_capability(Capability {
                stat: true,
                read: true,
                read_with_range: true,
                write: true,
                write_can_multi: true,
                write_can_empty: true,
                write_with_content_type: self.config.support_cty,
                create_dir: true,
                delete: true,
                list: true,
                ..Default::default()
            })
            .set_root("/");
        info
    }

    async fn stat(&self, path: &str, _: OpStat) -> Result<RpStat> {
        let path = NormalPath::try_new(path)?;
        let metadata = self
            .state
            .get(&path)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Not found"))?
            .metadata
            .clone();
        Ok(RpStat::new(metadata))
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        let path = NormalPath::try_new(path)?.assert_is_file_path()?;

        let state = self
            .state
            .get(&path)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Not found"))?;

        let bs = apply_range(state.content.clone(), args.range());

        Ok((
            RpRead::new().with_size(Some(state.metadata.content_length())).with_range(Some()),
            oio::Cursor::from(bs),
        ))
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Lister)> {
        if args.recursive() {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Recursive listing is not yet supported.",
            ));
        }

        let ns_path = NormalPath::try_new(path)?.assert_is_ns_path()?;

        todo!()
    }
}

/// Reader for [`InmemAccessor`].
pub struct InmemReader {
    content: EcoVec<Bytes>,
    range: BytesRange,
    cursor: u64,
}
