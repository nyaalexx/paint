use std::borrow::Cow;

use chrono::{DateTime, Utc};
use glam::UVec2;

#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub creation_time: DateTime<Utc>,
    pub modify_time: DateTime<Utc>,
    pub resolution: UVec2,
}

#[derive(Debug, Clone)]
pub struct Texture<'a> {
    pub resolution: UVec2,
    pub format: TextureFormat,
    pub data: Cow<'a, [u8]>,
    pub row_stride: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TextureFormat {
    Rgba8NonlinearSrgb,
}
