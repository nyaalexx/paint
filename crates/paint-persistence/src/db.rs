use std::path::Path;

use paint_core::persistence::{Error as ApiError, Metadata};
use rusqlite::{Connection, MAIN_DB, OpenFlags, OptionalExtension, TransactionBehavior};

use crate::metadata::VersionedMetadata;

const APP_ID: u32 = 0x3b796974;
const VERSION: u32 = 1;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(ApiError);

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Self(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self(ApiError::Internal(Box::new(err)))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self(ApiError::Internal(Box::new(err)))
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        match err.sqlite_error_code() {
            Some(rusqlite::ErrorCode::CannotOpen) => Self(ApiError::FileInaccessible),
            Some(rusqlite::ErrorCode::NotADatabase) => Self(ApiError::UnknownFormat),
            _ => Self(ApiError::Internal(Box::new(err))),
        }
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        err.0
    }
}

#[derive(Debug)]
pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn create_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        Self::init_db(&mut conn)?;
        Ok(Self { conn })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let mut conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let is_init = Self::is_db_init(&mut conn)?;
        if !is_init {
            return Err(ApiError::UnknownFormat.into());
        }

        Ok(Self { conn })
    }

    fn init_db(conn: &mut Connection) -> Result<()> {
        let sql = r"
            PRAGMA locking_mode = EXCLUSIVE;
            PRAGMA journal_mode = DELETE;
        ";
        conn.execute_batch(sql)?;

        let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;

        // unfortunately pragma statements do not support parameters
        let sql = format!("PRAGMA application_id = {APP_ID}");
        tx.execute(&sql, [])?;

        let sql = format!("PRAGMA user_version = {VERSION}");
        tx.execute(&sql, [])?;

        let sql = r"
            CREATE TABLE metadata (json TEXT);
            CREATE TABLE texture (data BLOB);
        ";
        tx.execute_batch(sql)?;

        tx.commit()?;

        Ok(())
    }

    fn is_db_init(conn: &mut Connection) -> Result<bool> {
        let sql = "SELECT application_id FROM pragma_application_id";
        let app_id: u32 = conn.query_one(sql, [], |row| row.get(0))?;
        if app_id != APP_ID {
            return Ok(false);
        }

        let sql = "SELECT user_version FROM pragma_user_version";
        let version: u32 = conn.query_one(sql, [], |row| row.get(0))?;
        if version != VERSION {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn get_metadata(&self) -> Result<Option<Metadata>> {
        let sql = "SELECT json FROM metadata";
        let opt_json: Option<String> = self.conn.query_row(sql, [], |row| row.get(0)).optional()?;
        let Some(json) = opt_json else {
            return Ok(None);
        };

        let metadata = serde_json::from_str::<VersionedMetadata>(&json)?;
        Ok(Some(metadata.into()))
    }

    pub fn get_texture_data(&self) -> Result<Option<Vec<u8>>> {
        let sql = "SELECT rowid FROM texture";
        let opt_id: Option<i64> = self.conn.query_row(sql, [], |row| row.get(0)).optional()?;
        let Some(id) = opt_id else {
            return Ok(None);
        };

        let read_only = true;
        let blob = self
            .conn
            .blob_open(MAIN_DB, "texture", "data", id, read_only)?;

        let mut data = vec![0; blob.len()];
        blob.read_at(&mut data, 0)?;

        Ok(Some(data))
    }

    pub fn begin_change(&mut self) -> Result<Transaction<'_>> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Exclusive)?;
        Ok(Transaction { tx })
    }

    pub fn save_copy(&self, path: &Path) -> Result<()> {
        if path.exists() {
            return Err(ApiError::FileExists.into());
        }

        let Some(target_dir) = path.parent() else {
            return Err(ApiError::FileInaccessible.into());
        };

        let temp_file = tempfile::Builder::new()
            .prefix(".paint-temp-")
            .tempfile_in(target_dir)?;

        let Some(utf8_path) = temp_file.path().as_os_str().to_str() else {
            return Err(ApiError::FileInaccessible.into());
        };

        let sql = "VACUUM INTO ?1";
        self.conn.execute(sql, [utf8_path])?;

        temp_file.persist_noclobber(path).map_err(|e| e.error)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Transaction<'a> {
    tx: rusqlite::Transaction<'a>,
}

impl Transaction<'_> {
    pub fn set_metadata(&mut self, metadata: Metadata) -> Result<()> {
        let metadata = VersionedMetadata::from(metadata);
        let json = serde_json::to_string(&metadata)?;

        let sql = "DELETE FROM metadata";
        self.tx.execute(sql, [])?;

        let sql = "INSERT INTO metadata VALUES (?1)";
        self.tx.execute(sql, [json])?;

        Ok(())
    }

    pub fn set_texture_data(&mut self, data: &[u8]) -> Result<()> {
        let sql = "DELETE FROM texture";
        self.tx.execute(sql, [])?;

        let sql = "INSERT INTO texture VALUES (ZEROBLOB(?1))";
        self.tx.execute(sql, [data.len() as i64])?;

        let sql = "SELECT rowid FROM texture";
        let id: i64 = self.tx.query_row(sql, [], |row| row.get(0))?;

        let read_only = false;
        let mut blob = self
            .tx
            .blob_open(MAIN_DB, "texture", "data", id, read_only)?;

        blob.write_at(data, 0)?;

        Ok(())
    }

    pub fn commit(self) -> Result<()> {
        self.tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use glam::UVec2;

    use super::*;

    #[test]
    fn open() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join("file");

        let db = Db::create_in_memory()?;
        db.save_copy(&path)?;
        drop(db);

        let db = Db::open(&path)?;
        assert!(db.get_metadata()?.is_none());

        Ok(())
    }

    #[test]
    fn open_err_inaccessible() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join("no_such_file");

        let err = Db::open(&path).err().unwrap();
        assert!(matches!(err, Error(ApiError::FileInaccessible)));

        Ok(())
    }

    #[test]
    fn open_err_unknown_format() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join("no_such_file");

        std::fs::write(&path, "something")?;

        let err = Db::open(&path).err().unwrap();
        assert!(matches!(err, Error(ApiError::UnknownFormat)));

        Ok(())
    }

    #[test]
    fn set_get_metadata() -> Result<()> {
        let tempdir = tempfile::tempdir()?;
        let path = tempdir.path().join("file");

        let db = Db::create_in_memory()?;
        db.save_copy(&path)?;
        drop(db);
        let mut db = Db::open(&path)?;

        let metadata = Metadata {
            creation_time: Utc::now(),
            change_time: Utc::now(),
            resolution: UVec2::new(128, 128),
        };

        let mut tx = db.begin_change()?;
        tx.set_metadata(metadata.clone())?;
        tx.commit()?;

        let got_metadata = db.get_metadata()?.unwrap();
        assert!(metadata.approx_eq(&got_metadata));

        drop(db);
        let db = Db::open(&path)?;
        let got_metadata = db.get_metadata()?.unwrap();
        assert!(metadata.approx_eq(&got_metadata));

        Ok(())
    }
}
