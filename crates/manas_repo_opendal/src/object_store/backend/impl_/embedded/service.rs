//! I define an opendal service interface over binary
//! embedded directories.
//!

// NOTE: rust-embed returns root-less paths of assets.
// It doesn't include any dir paths in assets.

use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{LocalResult, TimeZone, Utc};
use either::Either;
use manas_http::header::common::media_type::MediaType;
use opendal::{
    raw::{
        oio::{self, Entry, Page},
        Accessor, AccessorInfo, BytesRange, OpList, OpRead, OpStat, RpList, RpRead, RpStat,
    },
    Builder, Capability, EntryMode, Error, ErrorKind, Metadata, Result, Scheme,
};
use rust_embed::RustEmbed;

use crate::object_store::backend::impl_::common::service_object_path::{
    ClassifiedPath, FilePath, NormalPath, NsPath,
};

/// An implementation of builder for opendal service that
/// reads objects from binary embedded directory.
#[derive(Debug)]
pub struct Embedded<Assets> {
    name: Option<String>,
    _phantom: PhantomData<fn() -> Assets>,
}

impl<Assets> Default for Embedded<Assets> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Assets> Embedded<Assets> {
    /// Get a new [`Embedded`] service builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            name: None,
            _phantom: PhantomData,
        }
    }

    /// Get builder set with given name.
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

impl<Assets: RustEmbed + 'static> Builder for Embedded<Assets> {
    const SCHEME: Scheme = Scheme::Custom("Embedded");

    type Accessor = EmbeddedAccessor<Assets>;

    fn from_map(mut map: HashMap<String, String>) -> Self {
        Self {
            name: map.remove("name"),
            ..Default::default()
        }
    }

    fn build(&mut self) -> Result<Self::Accessor> {
        self.name
            .take()
            .map(|name| EmbeddedAccessor::new(name))
            .ok_or_else(|| Error::new(ErrorKind::ConfigInvalid, "No name specified."))
    }
}

/// An implementation of opendal service that reads objects
/// from binary embedded directory.
#[derive(Clone)]
pub struct EmbeddedAccessor<Assets: RustEmbed> {
    name: String,
    _phantom: PhantomData<fn() -> Assets>,
}

impl<Assets: RustEmbed> std::fmt::Debug for EmbeddedAccessor<Assets> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Embedded")
            .field("name", &self.name)
            .finish()
    }
}

impl<Assets: RustEmbed> EmbeddedAccessor<Assets> {
    /// Get a new [`Embedded`] service with given name.
    #[inline]
    pub fn new(name: String) -> Self {
        Self {
            name,
            _phantom: PhantomData,
        }
    }
}

fn apply_range(mut bs: Bytes, br: BytesRange) -> Bytes {
    match (br.offset(), br.size()) {
        (Some(offset), Some(size)) => {
            let mut bs = bs.split_off(offset as usize);
            if (size as usize) < bs.len() {
                let _ = bs.split_off(size as usize);
            }
            bs
        }
        (Some(offset), None) => bs.split_off(offset as usize),
        (None, Some(size)) => bs.split_off(bs.len() - size as usize),
        (None, None) => bs,
    }
}

impl<Assets: RustEmbed + 'static> EmbeddedAccessor<Assets> {
    fn ns_info(path: &NsPath<'_>) -> Option<Metadata> {
        Assets::iter()
            .any(|f| f.starts_with(path.as_str()))
            .then(|| Metadata::new(EntryMode::DIR))
    }

    fn file_info(path: &FilePath<'_>) -> Option<(Metadata, Cow<'static, [u8]>)> {
        let file = Assets::get(path)?;

        let f_metadata = file.metadata;

        let mut metadata = Metadata::new(EntryMode::FILE)
            .with_content_length(file.data.len() as u64)
            .with_content_type(
                MediaType::guess_from_path(path)
                    .unwrap_or_default()
                    .to_string(),
            )
            .with_etag(hex::encode(f_metadata.sha256_hash()));

        if let Some(timestamp) = f_metadata.last_modified() {
            if let LocalResult::Single(lmd) = Utc.timestamp_opt(timestamp as i64, 0) {
                metadata.set_last_modified(lmd);
            }
        }

        Some((metadata, file.data))
    }
}

#[async_trait]
impl<Assets: RustEmbed + 'static> Accessor for EmbeddedAccessor<Assets> {
    type Reader = oio::Cursor;
    type BlockingReader = ();
    type Writer = ();
    type Appender = ();
    type BlockingWriter = ();
    type Pager = NsPage<Assets>;
    type BlockingPager = ();

    fn info(&self) -> AccessorInfo {
        let mut info = AccessorInfo::default();
        info.set_scheme(opendal::Scheme::Custom("Embedded"))
            .set_name(self.name.as_str())
            .set_capability(Capability {
                stat: true,
                read: true,
                read_with_range: true,
                list: true,
                list_with_delimiter_slash: true,

                ..Default::default()
            })
            .set_root("/");
        info
    }

    async fn stat(&self, path: &str, _: OpStat) -> Result<RpStat> {
        let path = ClassifiedPath::new(NormalPath::try_new(path)?);

        let metadata = match path.0 {
            Either::Left(ns_path) => Self::ns_info(&ns_path),
            Either::Right(file_path) => Self::file_info(&file_path).map(|(metadata, _)| metadata),
        }
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "Not found"))?;

        Ok(RpStat::new(metadata))
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        let path = NormalPath::try_new(path)?.assert_is_file_path()?;

        let (mut metadata, data) = Self::file_info(&path)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "File not found"))?;

        // TODO no clone if static bytes.
        let bs = apply_range(data.clone().into_owned().into(), args.range());
        metadata = metadata.with_content_length(bs.len() as u64);
        // TODO content-range should be included.

        Ok((RpRead::with_metadata(metadata), oio::Cursor::from(bs)))
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Pager)> {
        if args.delimiter() != "/" {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Non slash delimiter is not yet supported.",
            ));
        }

        let ns_path = NormalPath::try_new(path)?
            .into_owned()
            .assert_is_ns_path()?;

        Ok((
            Default::default(),
            NsPage::<Assets> {
                ns_path,
                is_exhausted: false,
                _phantom: PhantomData,
            },
        ))
    }
}

/// Pager for [`Embedded`] service.
pub struct NsPage<Assets> {
    ns_path: NsPath<'static>,
    is_exhausted: bool,
    _phantom: PhantomData<fn() -> Assets>,
}

#[async_trait]
impl<Assets: RustEmbed + 'static> Page for NsPage<Assets> {
    async fn next(&mut self) -> Result<Option<Vec<Entry>>> {
        Ok((!self.is_exhausted).then(|| {
            Assets::iter()
                .filter_map(|path| {
                    let is_child = path
                        .rsplit_once('/')
                        .map(|(prefix, name)| {
                            prefix == self.ns_path.trim_end_matches('/') && !name.is_empty()
                        })
                        .unwrap_or_default();

                    if !is_child {
                        return None;
                    }

                    let path = ClassifiedPath::new(NormalPath::try_new(path.as_ref()).ok()?);

                    let metadata = match &path.0 {
                        Either::Left(ns_path) => EmbeddedAccessor::<Assets>::ns_info(ns_path),
                        Either::Right(file_path) => {
                            EmbeddedAccessor::<Assets>::file_info(file_path)
                                .map(|(metadata, _)| metadata)
                        }
                    }?;

                    Some(Entry::new(path.as_str(), metadata))
                })
                .collect()
        }))
    }
}

// #[derive(RustEmbed)]
// #[folder = "../../fcrates/rdf_dynsyn/"]
// // #[prefix = "prefix/"]
// struct TestAssets;

// #[test]
// fn test() {
//     for a in TestAssets::iter() {
//         println!("{}", a);
//     }

//     println!(
//         "{:?}",
//         TestAssets::get("src").map(|f| (f.metadata.last_modified(), f.metadata.sha256_hash()))
//     );
//     println!(
//         "{:?}",
//         TestAssets::get("Cargo.toml")
//             .map(|f| (f.metadata.last_modified(), f.metadata.sha256_hash()))
//     );
//     println!(
//         "{:?}",
//         TestAssets::get("src").map(|f| (f.metadata.last_modified(), f.metadata.sha256_hash()))
//     );
//     println!(
//         "{:?}",
//         TestAssets::get("src/").map(|f| (f.metadata.last_modified(), f.metadata.sha256_hash()))
//     );
// }
