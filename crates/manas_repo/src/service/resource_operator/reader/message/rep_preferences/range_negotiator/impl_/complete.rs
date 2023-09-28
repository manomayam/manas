use manas_http::representation::metadata::RepresentationMetadata;

use crate::service::resource_operator::reader::message::rep_preferences::range_negotiator::RangeNegotiator;

/// An implementation of [`RangeNegotiator`], that always negotiate for complete representation.
#[derive(Debug, Clone, Default)]
pub struct CompleteRangeNegotiator;

impl RangeNegotiator for CompleteRangeNegotiator {
    #[inline]
    fn resolve_pref_range(
        self: Box<Self>,
        _rep_metadata: &RepresentationMetadata,
    ) -> Option<headers::Range> {
        None
    }
}
