//! I define type to represent in-memory bytes data.
//!

use std::io::{Read, Write};

use bytes::Bytes;
use ecow::{eco_vec, EcoVec};

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
    pos: usize,
}

impl BytesInmemReader {
    /// Create a new [`BytesInmemReader`].
    pub fn new(data: EcoVec<Bytes>) -> Self {
        Self { data, pos: 0 }
    }
}

impl Read for BytesInmemReader {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(chunk) = self.data.get(self.pos) {
            buf.write_all(chunk)?;
            self.pos += 1;
            Ok(chunk.len())
        } else {
            Ok(0)
        }
    }
}
