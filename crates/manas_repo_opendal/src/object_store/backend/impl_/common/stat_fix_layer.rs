//! I provide an opendal service layer for fixing incorrect stat
//! behavior on various cloud backends. (<https://github.com/apache/incubator-opendal/issues/3199>)

use futures::FutureExt;
use once_cell::sync::Lazy;
use opendal::raw::Access;

use opendal::raw::*;
use opendal::*;

// An opendal service layer for fixing incorrect stat
/// behavior on various cloud backends. (https://github.com/apache/incubator-opendal/issues/3199)
#[derive(Debug)]
pub struct StatFixAccess<A: Access> {
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

impl<A: Access> LayeredAccess for StatFixAccess<A> {
    type Inner = A;
    type Reader = A::Reader;
    type BlockingReader = A::BlockingReader;
    type Writer = A::Writer;
    type BlockingWriter = A::BlockingWriter;
    type Lister = A::Lister;
    type BlockingLister = A::BlockingLister;
    type Deleter = A::Deleter;
    type BlockingDeleter = A::BlockingDeleter;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn stat(&self, path: &str, args: OpStat) -> impl Future<Output = Result<RpStat>> + MaybeSend {
        self.inner()
            .stat(path, args)
            .map(move |resp| fix_stat_response(path, resp))
    }

    #[inline]
    fn read(
        &self,
        path: &str,
        args: OpRead,
    ) -> impl Future<Output = Result<(RpRead, Self::Reader)>> + MaybeSend {
        self.inner.read(path, args)
    }

    #[inline]
    fn write(
        &self,
        path: &str,
        args: OpWrite,
    ) -> impl Future<Output = Result<(RpWrite, Self::Writer)>> + MaybeSend {
        self.inner.write(path, args)
    }

    #[inline]
    fn delete(&self) -> impl Future<Output = Result<(RpDelete, Self::Deleter)>> + MaybeSend {
        self.inner.delete()
    }

    #[inline]
    fn list(
        &self,
        path: &str,
        args: OpList,
    ) -> impl Future<Output = Result<(RpList, Self::Lister)>> + MaybeSend {
        self.inner.list(path, args)
    }

    #[inline]
    fn blocking_read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::BlockingReader)> {
        self.inner.blocking_read(path, args)
    }

    #[inline]
    fn blocking_write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::BlockingWriter)> {
        self.inner.blocking_write(path, args)
    }

    #[inline]
    fn blocking_delete(&self) -> Result<(RpDelete, Self::BlockingDeleter)> {
        self.inner.blocking_delete()
    }

    #[inline]
    fn blocking_list(&self, path: &str, args: OpList) -> Result<(RpList, Self::BlockingLister)> {
        self.inner.blocking_list(path, args)
    }

    fn blocking_stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        fix_stat_response(path, self.inner().blocking_stat(path, args))
    }
}

/// A layer to wrap with [`StatFixAccess`].
pub struct StatFixLayer;

impl<A: Access> Layer<A> for StatFixLayer {
    type LayeredAccess = StatFixAccess<A>;

    fn layer(&self, inner: A) -> Self::LayeredAccess {
        StatFixAccess { inner }
    }
}
