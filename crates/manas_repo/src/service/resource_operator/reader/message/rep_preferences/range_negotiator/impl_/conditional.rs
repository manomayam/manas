use headers::{IfRange, Range};
use manas_http::representation::metadata::{KDerivedETag, KLastModified, RepresentationMetadata};
use typed_record::TypedRecord;

use crate::service::resource_operator::reader::message::rep_preferences::range_negotiator::RangeNegotiator;

/// An implementation of [`RangeNegotiator`], that
/// resolves it's preferred range based on given conditional
/// range headers.
#[derive(Debug, Clone)]
pub struct ConditionalRangeNegotiator {
    /// Preferred range, if condition matches.
    pub range: Option<Range>,

    /// If-Range precondition.
    pub if_range: Option<IfRange>,
}

impl RangeNegotiator for ConditionalRangeNegotiator {
    fn resolve_pref_range(
        self: Box<Self>,
        selected_rep_metadata: &RepresentationMetadata,
    ) -> Option<Range> {
        let this = *self;
        this.range.and_then(|range| {
            // If there is a if-Range precondition
            if let Some(if_range) = this.if_range {
                let last_modified = selected_rep_metadata.get_rv::<KLastModified>();
                let etag = selected_rep_metadata
                    .get_rv::<KDerivedETag>()
                    .map(|detag| detag.as_ref());

                // If resource rep is modified, request full representation.
                // TODO should compare against base etag?
                if if_range.is_modified(etag, last_modified) {
                    return None;
                }
            }
            Some(range)
        })
    }
}
