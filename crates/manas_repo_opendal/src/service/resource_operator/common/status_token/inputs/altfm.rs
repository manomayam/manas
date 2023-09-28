//! I define types for representing
//! fat objects of alt-metadata of an object.
//!

use manas_http::header::common::media_type::MediaType;
use opendal::Metadata;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::supplem_fat_data::{SupplemFatData, VersionPinnedSupplemData};

/// A struct representing alt-metadata.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AltMetadata {
    /// Content-type of the alt representation.
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub content_type: Option<MediaType>,
}

/// Type alias for alt fat metadata.
pub type AltFatMetadata = SupplemFatData<AltMetadata>;

impl AltFatMetadata {
    /// Resolve a new alt fat metadata object from given params.
    pub fn resolve_new(
        prev_alt_metadata: Option<AltMetadata>,
        prev_obj_metadata: Option<Metadata>,
        new_content_type: MediaType,
    ) -> Self {
        // Construct alt metadata backup.
        let prev_alt_metadata_backup = prev_obj_metadata.and_then(|m| {
            // Backup is defined, only if main object has a timestamp.
            m.last_modified().and_then(|lm| {
                prev_alt_metadata
                    .clone()
                    .map(|alt_metadata| VersionPinnedSupplemData {
                        main_obj_etag: m.etag().map(|etag| etag.to_owned()),
                        main_obj_timestamp: lm,
                        data: alt_metadata,
                    })
            })
        });

        let mut live_alt_metadata = prev_alt_metadata.unwrap_or_default();
        live_alt_metadata.content_type = Some(new_content_type);

        Self {
            live: live_alt_metadata,
            prev_backup: prev_alt_metadata_backup,
        }
    }
}
