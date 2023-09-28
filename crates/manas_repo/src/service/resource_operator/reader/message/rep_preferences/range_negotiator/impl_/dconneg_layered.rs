use std::fmt::Debug;

use manas_http::{
    header::common::media_type::MediaType, representation::metadata::RepresentationMetadata,
};

use crate::service::resource_operator::reader::message::rep_preferences::range_negotiator::{
    DynRangeNegotiator, RangeNegotiator,
};

/// An implementation of [`RangeNegotiator`] which
///
/// - resolves complete range as preferred range if
/// negotiated content-type is different from that of source
/// rep content-type.
///
/// - Delegates to outer negotiator otherwise.
///

#[derive(Debug, Clone)]
pub struct DConnegLayeredRangeNegotiator<CN> {
    /// Outer range negotiator.
    pub outer: Box<DynRangeNegotiator>,

    /// Derived Content type negotiator.
    pub dconneger: CN,
}

impl<CN> RangeNegotiator for DConnegLayeredRangeNegotiator<CN>
where
    CN: DContentTypeNegotiator,
{
    fn resolve_pref_range(
        self: Box<Self>,
        rep_metadata: &RepresentationMetadata,
    ) -> Option<headers::Range> {
        // Get negotiated content-type.
        let negotiated_content_type = self
            .dconneger
            .resolve_pref_derived_content_type(rep_metadata.content_type());

        // If negotiated content-type is same as that of source rep,
        // then delegate to outer negotiator.
        if &negotiated_content_type == rep_metadata.content_type() {
            self.outer.resolve_pref_range(rep_metadata)
        }
        // Else resolve for complete rep.
        else {
            None
        }
    }
}

/// A trait for defining derived content-type negotiators.
pub trait DContentTypeNegotiator: Debug + Clone + Send + 'static {
    /// Resolve preferred content-type for given content-type
    /// of base representation.
    fn resolve_pref_derived_content_type(self, base_content_type: &MediaType) -> MediaType;
}
