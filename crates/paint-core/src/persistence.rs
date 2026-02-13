use std::borrow::Cow;
use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};
use glam::UVec2;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("file already exists, refusing to overwrite")]
    FileExists,
    #[error("file could not be accessed")]
    FileInaccessible,
    #[error("file format could not be recognized")]
    UnknownFormat,
    #[error("internal error")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn internal<E>(error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error::Internal(error.into())
    }
}

pub trait Project: std::fmt::Debug + Sized + 'static {
    type Transaction<'a>: Transaction;

    fn open(path: &Path) -> Result<Self>;

    fn create(new_project: NewProject) -> Result<Self>;

    fn path(&self) -> Option<&Path>;

    fn get_metadata(&self) -> Result<Metadata>;

    fn get_texture(&self) -> Result<Texture<'static>>;

    fn save_copy(&self, new_path: &Path) -> Result<Self>;

    fn begin_change(&mut self) -> Result<Self::Transaction<'_>>;
}

pub trait Transaction: std::fmt::Debug + Sized {
    fn update_metadata<F>(&mut self, func: F) -> Result<()>
    where
        F: FnOnce(&mut Metadata);

    fn set_texture(&mut self, new_texture: &Texture<'_>) -> Result<()>;

    fn commit(self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct NewProject {
    pub resolution: UVec2,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub creation_time: DateTime<Utc>,
    pub change_time: DateTime<Utc>,
    pub resolution: UVec2,
}

impl Metadata {
    pub fn approx_eq(&self, other: &Metadata) -> bool {
        ((self.creation_time - other.creation_time).abs() <= TimeDelta::seconds(1))
            && ((self.change_time - other.change_time).abs() <= TimeDelta::seconds(1))
            && (self.resolution == other.resolution)
    }
}

#[derive(Debug, Clone)]
pub struct Texture<'a> {
    pub resolution: UVec2,
    pub format: TextureFormat,
    pub data: Cow<'a, [u8]>,
    pub row_stride: usize,
}

impl Texture<'_> {
    pub fn new_white(resolution: UVec2) -> Texture<'static> {
        Texture {
            resolution,
            format: TextureFormat::Rgba8NonlinearSrgb,
            data: Cow::Owned(vec![
                255;
                4 * (resolution.x as usize) * (resolution.y as usize)
            ]),
            row_stride: (resolution.x as usize) * 4,
        }
    }

    pub fn make_owned(&self) -> Texture<'static> {
        Texture {
            resolution: self.resolution,
            format: self.format,
            data: self.data.clone().into_owned().into(),
            row_stride: self.row_stride,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum TextureFormat {
    Rgba8NonlinearSrgb,
}
