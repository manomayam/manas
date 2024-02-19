//! I define type to represent in-memory bytes data.
//!

use std::{
    io::{self, Cursor, Read},
    task::Poll,
};

use bytes::Bytes;
use ecow::{eco_vec, EcoVec};
use futures::AsyncRead;
use http_typed_headers::range::{BytesRange, BytesRangeValueSpec, RangeUnsatisfiableError};
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

    /// Get the reader.
    #[inline]
    pub fn reader(&self) -> BytesInmemReader {
        BytesInmemReader::new(self.clone())
    }

    /// Get the reader for specified range.
    #[inline]
    pub fn range_reader(
        &self,
        range_spec: BytesRangeValueSpec,
    ) -> Result<BytesInmemReader, RangeUnsatisfiableError> {
        BytesInmemReader::new_with_range(self.clone(), range_spec)
    }
}

/// A reader over slice of bytes chunks.
#[derive(Debug, Clone)]
pub struct BytesInmemReader {
    data: EcoVec<Bytes>,
    range_bounds: Option<((usize, usize), (usize, usize))>,
    iter_next_ch_index: Option<usize>,
    iter_ch_cursor: Option<Cursor<Bytes>>,
}

impl BytesInmemReader {
    /// Create a new [`BytesInmemReader`].
    pub fn new(bytes_inmem: BytesInmem) -> Self {
        let mut data = bytes_inmem.bytes;
        // Remove empty chunks.
        data.retain(|chunk| !chunk.is_empty());

        Self {
            range_bounds: data
                .last()
                .map(|last_ch| ((0, 0), (data.len() - 1, last_ch.len() - 1))),
            data,
            iter_next_ch_index: Some(0),
            iter_ch_cursor: None,
        }
    }

    /// Create a new [`BytesInmemReader`] over a range in inmem bytes data.
    pub fn new_with_range(
        bytes_inmem: BytesInmem,
        range_spec: BytesRangeValueSpec,
    ) -> Result<Self, RangeUnsatisfiableError> {
        let range = BytesRange::new(&range_spec, &[]);

        // Resolve the effective range.
        let effective_range = range
            .validate(bytes_inmem.size)?
            .pop()
            .expect("Must be Some");

        let mut data = bytes_inmem.bytes;
        // Remove empty chunks.
        data.retain(|chunk| !chunk.is_empty());

        // Resolve range indices as tuple of (chunk_index, offset_index).
        let mut range_start_index = (0, 0);
        let mut range_end_index = (
            data.len() - 1,
            data.last()
                .expect("Must be some if range is satisfiable.")
                .len()
                - 1,
        );

        let mut cum_len = 0;
        for (i, ch) in data.iter().enumerate() {
            let next_cum_len = cum_len + ch.len() as u64;
            if cum_len - 1 < *effective_range.start()
                && next_cum_len - 1 >= *effective_range.start()
            {
                range_start_index = (i, (effective_range.start() - cum_len) as usize);
            }

            if cum_len - 1 < *effective_range.end() && next_cum_len - 1 >= *effective_range.end() {
                range_end_index = (i, (effective_range.end() - cum_len) as usize);
                break;
            }

            cum_len = next_cum_len;
        }

        Ok(Self {
            data,
            iter_next_ch_index: Some(range_start_index.0),
            iter_ch_cursor: None,
            range_bounds: Some((range_start_index, range_end_index)),
        })
    }
}

impl Read for BytesInmemReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let range_bounds = if let Some(bounds) = self.range_bounds.as_ref() {
            bounds
        } else {
            // No data.
            return Ok(0);
        };

        // Resolve the cursor for this read.
        let mut ch_cursor = if let Some(c) = self.iter_ch_cursor.take() {
            c
        } else if let Some(ch_index) = self.iter_next_ch_index {
            self.iter_next_ch_index = Some(ch_index + 1);
            let mut chunk = self.data[ch_index].clone();
            // If chunk is range end chunk
            if ch_index == range_bounds.1 .0 {
                chunk = chunk.split_to(range_bounds.1 .1 + 1);
                self.iter_next_ch_index = None;
            }
            // If chunk is range start chunk
            if ch_index == range_bounds.0 .0 {
                chunk = chunk.split_off(range_bounds.0 .1);
            }
            Cursor::new(chunk)
        } else {
            return Ok(0);
        };

        // Read from the resolved cursor.
        let count = ch_cursor.read(buf).map_err(|e| {
            error!("Error in reading to buffer. ${e}");
            e
        })?;

        // If cursor still has remaining slice, then store it back for next read.
        if ch_cursor.position() < ch_cursor.get_ref().len() as u64 {
            self.iter_ch_cursor = Some(ch_cursor);
        }

        Ok(count)
    }
}

impl AsyncRead for BytesInmemReader {
    #[inline]
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(io::Read::read(self.get_mut(), buf))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use bytes::Bytes;
    use ecow::EcoVec;
    use http_typed_headers::range::BytesRangeValueSpec;
    use rstest::rstest;

    use super::BytesInmem;

    fn bim(data: &[&str]) -> BytesInmem {
        BytesInmem::from(
            data.iter()
                .map(|s| Bytes::from(s.as_bytes().to_vec()))
                .collect::<EcoVec<_>>(),
        )
    }

    #[rstest]
    #[case(bim(&[]), "")]
    #[case(bim(&["Rama is ", "Manifestation", " of ", "", "dharma"]), "Rama is Manifestation of dharma")]
    fn full_reader_works_correctly(#[case] bytes_inmem: BytesInmem, #[case] expected: &str) {
        let mut reader = bytes_inmem.reader();
        let mut buf = String::new();
        reader.read_to_string(&mut buf).expect("Must be valid");
        assert_eq!(buf, expected);
    }

    // #[rstest]
    // #[case(bim(&["Rama is ", "Manifestation", " of ", "", "dharma"])), &[
    //     // (BytesRangeValueSpec::)
    // ]]
    // fn range_reader_works_correctly(
    //     #[case] bytes_inmem: BytesInmem,
    //     #[case] expected: &[(BytesRangeValueSpec, &str)],
    // ) {
    //     for (range_spec, expected_str) in expected {
    //         let mut reader = bytes_inmem
    //             .range_reader(range_spec.clone())
    //             .expect("Claimed valid range.");
    //         let mut buf = String::new();
    //         reader.read_to_string(&mut buf).expect("Must be valid");
    //         assert_eq!(buf, *expected_str);
    //     }
    // }
}
