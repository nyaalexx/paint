mod db;
mod metadata;
mod texture;

use std::path::{Path, PathBuf};

use chrono::Utc;
use paint_core::persistence::{Error, Metadata, NewProject, Result, Texture};

#[derive(Debug)]
pub struct Project {
    db: db::Db,
    path: Option<PathBuf>,
}

impl paint_core::persistence::Project for Project {
    type Transaction<'a> = Transaction<'a>;

    fn open(path: &Path) -> Result<Self> {
        let db = db::Db::open(path)?;

        if db.get_metadata()?.is_none() {
            return Err(Error::UnknownFormat);
        }

        Ok(Self {
            db,
            path: Some(path.into()),
        })
    }

    fn create(new_project: NewProject) -> Result<Self> {
        let mut db = db::Db::create_in_memory()?;

        let mut tx = db.begin_change()?;

        tx.set_metadata(Metadata {
            creation_time: Utc::now(),
            change_time: Utc::now(),
            resolution: new_project.resolution,
        })?;

        let texture_data = texture::encode(&Texture::new_white(new_project.resolution));
        tx.set_texture_data(&texture_data)?;

        tx.commit()?;

        Ok(Self { db, path: None })
    }

    fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn get_metadata(&self) -> Result<Metadata> {
        self.db.get_metadata()?.ok_or(Error::UnknownFormat)
    }

    fn get_texture(&self) -> Result<Texture<'static>> {
        let data = self.db.get_texture_data()?.ok_or(Error::UnknownFormat)?;
        texture::decode(&data)
    }

    fn begin_change(&mut self) -> Result<Self::Transaction<'_>> {
        let metadata = self.db.get_metadata()?.ok_or(Error::UnknownFormat)?;
        let tx = self.db.begin_change()?;
        Ok(Transaction { metadata, tx })
    }

    fn save_copy(&self, new_path: &Path) -> Result<Self> {
        self.db.save_copy(new_path)?;
        Self::open(new_path)
    }
}

#[derive(Debug)]
pub struct Transaction<'a> {
    metadata: Metadata,
    tx: db::Transaction<'a>,
}

impl paint_core::persistence::Transaction for Transaction<'_> {
    fn update_metadata<F>(&mut self, func: F) -> Result<()>
    where
        F: FnOnce(&mut Metadata),
    {
        func(&mut self.metadata);
        Ok(())
    }

    fn set_texture(&mut self, new_texture: &Texture<'_>) -> Result<()> {
        let data = texture::encode(new_texture);
        self.tx.set_texture_data(&data)?;
        Ok(())
    }

    fn commit(mut self) -> Result<()> {
        self.tx.set_metadata(self.metadata)?;
        self.tx.commit()?;
        Ok(())
    }
}
