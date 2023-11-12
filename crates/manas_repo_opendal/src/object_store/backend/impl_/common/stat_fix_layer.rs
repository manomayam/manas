//! I provide an opendal service layer for fixing incorrect stat
//! behavior on various cloud backends. (<https://github.com/apache/incubator-opendal/issues/3199>)

use once_cell::sync::Lazy;
use opendal::raw::Accessor;

use async_trait::async_trait;
use opendal::raw::*;
use opendal::*;

// An opendal service layer for fixing incorrect stat
/// behavior on various cloud backends. (https://github.com/apache/incubator-opendal/issues/3199)
#[derive(Debug)]
pub struct StatFixAccessor<A: Accessor> {
    inner: A,
}

static INVALID_EMPTY_METADATA: Lazy<Metadata> = Lazy::new(|| Metadata::new(EntryMode::DIR));

fn fix_stat_response(path: &str, resp: Result<RpStat>) -> Result<RpStat> {
    let inner_metadata = resp?.into_metadata();

    if (inner_metadata == *INVALID_EMPTY_METADATA) && (path != "/") {
        return Err(Error::new(ErrorKind::NotFound, "Not found."));
    }

    Ok(RpStat::new(inner_metadata))
}

#[async_trait]
impl<A: Accessor> LayeredAccessor for StatFixAccessor<A> {
    type Inner = A;
    type Reader = A::Reader;
    type BlockingReader = A::BlockingReader;
    type Appender = A::Appender;
    type Writer = A::Writer;
    type BlockingWriter = A::BlockingWriter;
    type Pager = A::Pager;
    type BlockingPager = A::BlockingPager;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        self.inner.read(path, args).await
    }

    fn blocking_read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::BlockingReader)> {
        self.inner.blocking_read(path, args)
    }

    async fn write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        self.inner.write(path, args).await
    }

    async fn append(&self, path: &str, args: OpAppend) -> Result<(RpAppend, Self::Appender)> {
        self.inner.append(path, args).await
    }

    fn blocking_write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::BlockingWriter)> {
        self.inner.blocking_write(path, args)
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Pager)> {
        self.inner.list(path, args).await
    }

    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingPager)> {
        self.inner.blocking_list(path, args)
    }

    async fn stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        fix_stat_response(path, self.inner().stat(path, args).await)
    }

    fn blocking_stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        fix_stat_response(path, self.inner().blocking_stat(path, args))
    }
}

/// A layer to wrap with [`StatFixAccessor`].
pub struct StatFixLayer;

impl<A: Accessor> Layer<A> for StatFixLayer {
    type LayeredAccessor = StatFixAccessor<A>;

    fn layer(&self, inner: A) -> Self::LayeredAccessor {
        StatFixAccessor { inner }
    }
}
