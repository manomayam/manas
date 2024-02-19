use std::{cmp::min, ops::Bound};

use bytes::Bytes;
use opendal::raw::BytesRange;

pub fn apply_range(mut bs: Bytes, br: BytesRange) -> (Bytes, BytesRange) {
    let full_size = bs.len() as u64;
    // Initialize with full range.
    let mut range = 0..full_size;

    // Update from required.
    if let Some(offset) = br.offset() {
        range.start = min(offset, full_size);
    }
    if let Some(size) = br.size() {
        range.end = min(range.start + size, full_size);
    }

    (
        bs.slice(range.),
        BytesRange::new(Some(range.start), Some(range.end)),
    )
}
