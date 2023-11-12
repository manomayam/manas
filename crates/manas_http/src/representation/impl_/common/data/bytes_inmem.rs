//! I define type to represent in-memory bytes data.
//!

use std::io::{self, Cursor, Read};

use bytes::Bytes;
use ecow::{eco_vec, EcoVec};
use tracing::error;

/// Bytes inmem data.
#[derive(Debug, Clone, Default)]
pub struct BytesInmem {
    /// Inmem bytes.
    pub(super) bytes: EcoVec<Bytes>,

    /// Cached size of the bytes.
    pub(super) size: u64,
}

impl From<EcoVec<Bytes>> for BytesInmem {
    #[inline]
    fn from(bytes: EcoVec<Bytes>) -> Self {
        Self {
            size: bytes.iter().fold(0, |acc, e| acc + e.len() as u64),
            bytes,
        }
    }
}

impl From<Bytes> for BytesInmem {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        Self {
            size: bytes.len() as u64,
            bytes: eco_vec![bytes],
        }
    }
}

impl From<String> for BytesInmem {
    #[inline]
    fn from(v: String) -> Self {
        let bytes: Bytes = v.into();
        Self {
            size: bytes.len() as u64,
            bytes: eco_vec![bytes],
        }
    }
}

impl From<BytesInmem> for EcoVec<Bytes> {
    fn from(value: BytesInmem) -> Self {
        value.bytes
    }
}

impl BytesInmem {
    /// Get bytes.
    #[inline]
    pub fn bytes(&self) -> &EcoVec<Bytes> {
        &self.bytes
    }

    /// Get size of the bytes data.
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get as reader.
    #[inline]
    pub fn as_read(&self) -> BytesInmemReader {
        BytesInmemReader::new(self.bytes.clone())
    }
}

/// A reader over slice of bytes chunks.
#[derive(Debug, Clone)]
pub struct BytesInmemReader {
    data: EcoVec<Bytes>,
    ch_index: usize,
    ch_cursor: Option<Cursor<Bytes>>,
}

impl BytesInmemReader {
    /// Create a new [`BytesInmemReader`].
    pub fn new(mut data: EcoVec<Bytes>) -> Self {
        // Remove empty chunks.
        data.retain(|chunk| !chunk.is_empty());

        Self {
            data,
            ch_index: 0,
            ch_cursor: None,
        }
    }
}

impl Read for BytesInmemReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Resolve the cursor for this read.
        let mut ch_cursor = if let Some(c) = self.ch_cursor.take() {
            c
        } else if let Some(chunk) = self.data.get(self.ch_index) {
            self.ch_index += 1;
            Cursor::new(chunk.clone())
        } else {
            return Ok(0);
        };

        // Read from the resolved cursor.
        let count = ch_cursor.read(buf).map_err(|e| {
            error!("Error in reading to buffer. ${e}");
            e
        })?;

        // If cursor stull has remaining slice, then store it back for next read.
        if ch_cursor.position() < ch_cursor.get_ref().len() as u64 {
            self.ch_cursor = Some(ch_cursor);
        }

        Ok(count)
    }
}
