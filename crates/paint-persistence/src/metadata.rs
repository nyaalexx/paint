use chrono::{DateTime, Utc};
use glam::UVec2;
use paint_core::persistence::Metadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "v")]
pub enum VersionedMetadata {
    V0(MetadataV0),
}

impl From<VersionedMetadata> for Metadata {
    fn from(m: VersionedMetadata) -> Self {
        match m {
            VersionedMetadata::V0(v0) => v0.into(),
        }
    }
}

impl From<Metadata> for VersionedMetadata {
    fn from(m: Metadata) -> Self {
        Self::V0(m.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataV0 {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub creation_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub change_time: DateTime<Utc>,
    pub resolution: UVec2,
}

impl From<Metadata> for MetadataV0 {
    fn from(m: Metadata) -> Self {
        Self {
            creation_time: m.creation_time,
            change_time: m.change_time,
            resolution: m.resolution,
        }
    }
}

impl From<MetadataV0> for Metadata {
    fn from(m: MetadataV0) -> Self {
        Self {
            creation_time: m.creation_time,
            change_time: m.change_time,
            resolution: m.resolution,
        }
    }
}
