mod db;
mod metadata;
mod texture;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use paint_core::persistence::{Error, Metadata, Result, Texture};

#[derive(Debug)]
pub struct Project {
    db: Arc<Mutex<db::Db>>,
    path: Option<PathBuf>,
}

impl paint_core::persistence::Project for Project {
    type Transaction = Transaction;

    fn open(path: &Path) -> Result<Self> {
        let db = db::Db::open(path)?;

        if db.get_metadata()?.is_none() {
            return Err(Error::UnknownFormat);
        }

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            path: Some(path.into()),
        })
    }

    fn create(metadata: Metadata) -> Result<Self> {
        let mut db = db::Db::create_in_memory()?;

        let mut tx = db.begin_change()?;

        tx.set_metadata(metadata.clone())?;

        let texture_data = texture::encode(&Texture::new_white(metadata.resolution));
        tx.set_texture_data(&texture_data)?;

        tx.commit()?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            path: None,
        })
    }

    fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn get_metadata(&self) -> Result<Metadata> {
        let db = self.db.lock().unwrap();
        db.get_metadata()?.ok_or(Error::UnknownFormat)
    }

    fn load_texture(&self) -> Result<Texture<'static>> {
        let db = self.db.lock().unwrap();
        let data = db.get_texture_data()?.ok_or(Error::UnknownFormat)?;
        texture::decode(&data)
    }

    fn begin_change(&self) -> Result<Self::Transaction> {
        let mut db_guard = self.db.lock().unwrap();
        let metadata = db_guard.get_metadata()?.ok_or(Error::UnknownFormat)?;
        let tx = db_guard.begin_change()?;
        Ok(Transaction {
            metadata,
            // SAFETY: here we erase the lifetime to 'static. the actual data is stored inside Arc,
            // so we store a copy of said Arc
            tx: unsafe { std::mem::transmute::<db::Transaction<'_>, db::Transaction<'static>>(tx) },
            _db_guard: unsafe {
                std::mem::transmute::<
                    std::sync::MutexGuard<'_, db::Db>,
                    std::sync::MutexGuard<'static, db::Db>,
                >(db_guard)
            },
            _db: self.db.clone(),
        })
    }

    fn save_copy(&self, new_path: &Path) -> Result<Self> {
        let db = self.db.lock().unwrap();
        db.save_copy(new_path)?;
        drop(db);
        Self::open(new_path)
    }
}

#[derive(Debug)]
pub struct Transaction {
    metadata: Metadata,
    tx: db::Transaction<'static>,
    _db_guard: std::sync::MutexGuard<'static, db::Db>,
    _db: Arc<Mutex<db::Db>>,
}

impl paint_core::persistence::Transaction for Transaction {
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
