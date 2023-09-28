//! I define types for representing self-backing up
//! fat objects of supplementary data of an odr object.
//!

use chrono::{serde::ts_milliseconds, DateTime, Utc};
use opendal::Metadata;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// A struct representing version pinned supplem data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionPinnedSupplemData<Data> {
    /// ETag of the pinned against main object versin.
    pub main_obj_etag: Option<String>,

    /// Timestamp of the pinned against main object version.
    #[serde(with = "ts_milliseconds")]
    pub main_obj_timestamp: DateTime<Utc>,

    /// Data.
    pub data: Data,
}

/// A struct representing fat self-backing up
/// fat objects off supplementary data of an odr object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplemFatData<Data> {
    /// Live supplem data.
    pub live: Data,

    /// Optional backup of supplem data associated with previous version.
    pub prev_backup: Option<VersionPinnedSupplemData<Data>>,
}

impl<Data: Clone> SupplemFatData<Data> {
    /// Resolve effective supplem data for main object with given metadata.
    #[tracing::instrument(skip_all)]
    pub fn resolve_effective_supplem_data(
        &self,
        main_obj_metadata: &Metadata,
    ) -> Result<Option<Data>, SupplemFatDataResolutionError> {
        match &self.prev_backup {
            // If backup of previous supplem data exist,
            Some(prev_backup) => {
                warn!("Backup of previous supplem data exists for main object.");

                // Ensure, main object has time stamp.
                let main_obj_time_stamp = main_obj_metadata
                    .last_modified()
                    .ok_or(SupplemFatDataResolutionError::MainObjWithContentBackupHasNoTimestamp)?;

                if (main_obj_time_stamp == prev_backup.main_obj_timestamp)
                    && (main_obj_metadata.etag() == prev_backup.main_obj_etag.as_deref())
                {
                    // If equal, previous update op corresponding to
                    // data should had been failed, and
                    // Existing main object's supplem data is resolved to be backed up supplem data.
                    warn!("Supplem data resolved to be backed up one, instead of live.");
                    Ok(Some(prev_backup.data.clone()))
                } else {
                    // If not equal, implies live supplem data corresponds to existing main object.
                    info!("Supplem data resolved to be live one.");
                    Ok(Some(self.live.clone()))
                }
            }

            // If no backup exists, then return live version.
            None => Ok(Some(self.live.clone())),
        }
    }
}

/// An error type for errors in resolving effective supplem data.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SupplemFatDataResolutionError {
    /// Invalid fat supplem data.
    #[error("Invalid supplem fat data.")]
    InvalidSupplemFatData,

    /// Main object with content backup has no time stamp.
    #[error("Main object with content backup has no time stamp.")]
    MainObjWithContentBackupHasNoTimestamp,

    /// Backup has invalid timestamp.
    #[error("Backup has invalid timestamp.")]
    BackupHasInvalidTimestamp,
}
