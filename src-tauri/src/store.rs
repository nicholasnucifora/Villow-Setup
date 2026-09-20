use crate::{
    error::{Error, Result},
    model::Installation,
};
use fs2::FileExt;
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    time::Duration,
};
pub struct Store {
    root: PathBuf,
}
pub struct OperationLock {
    file: File,
}
impl Drop for OperationLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}
impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root).map_err(|_| Error::Storage)?;
        if std::fs::symlink_metadata(root)
            .map_err(|_| Error::Storage)?
            .file_type()
            .is_symlink()
        {
            return Err(Error::Storage);
        }
        let s = Self {
            root: root.to_path_buf(),
        };
        s.connection()?.execute_batch("CREATE TABLE IF NOT EXISTS state (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS trust_state (id INTEGER PRIMARY KEY CHECK(id=1), sequence INTEGER NOT NULL, digest TEXT NOT NULL);")
            .map_err(|_| Error::Storage)?;
        Ok(s)
    }
    fn connection(&self) -> Result<Connection> {
        let c = Connection::open(self.root.join("state.sqlite3")).map_err(|_| Error::Storage)?;
        c.busy_timeout(Duration::from_secs(2))
            .map_err(|_| Error::Storage)?;
        c.execute_batch("PRAGMA synchronous=FULL; PRAGMA journal_mode=DELETE;")
            .map_err(|_| Error::Storage)?;
        Ok(c)
    }
    pub fn lock(&self) -> Result<OperationLock> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root.join("operation.lock"))
            .map_err(|_| Error::Storage)?;
        file.try_lock_exclusive().map_err(|_| Error::Busy)?;
        Ok(OperationLock { file })
    }
    pub fn load(&self) -> Result<Option<Installation>> {
        let c = self.connection()?;
        let mut q = c
            .prepare("SELECT value FROM state WHERE id=1")
            .map_err(|_| Error::Storage)?;
        let mut rows = q.query([]).map_err(|_| Error::Storage)?;
        if let Some(row) = rows.next().map_err(|_| Error::Storage)? {
            let text: String = row.get(0).map_err(|_| Error::Storage)?;
            let s: Installation = serde_json::from_str(&text).map_err(|_| Error::Storage)?;
            if s.format != 1 {
                return Err(Error::Storage);
            }
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }
    pub fn save(&self, s: &Installation) -> Result<()> {
        let json = serde_json::to_string(s).map_err(|_| Error::Storage)?;
        self.connection()?.execute("INSERT INTO state(id,value) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET value=excluded.value", [json]).map_err(|_| Error::Storage)?;
        Ok(())
    }
    pub fn forget(&self) -> Result<()> {
        self.connection()?
            .execute_batch("PRAGMA secure_delete=ON; DELETE FROM state; VACUUM;")
            .map_err(|_| Error::Storage)
    }
    pub fn accept_release(&self, sequence: u64, digest: &str) -> Result<()> {
        let mut c = self.connection()?;
        let tx = c.transaction().map_err(|_| Error::Storage)?;
        let old: Option<(u64, String)> = tx
            .query_row(
                "SELECT sequence,digest FROM trust_state WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|_| Error::Storage)?;
        if let Some((seq, hash)) = old {
            if sequence < seq || (sequence == seq && digest != hash) {
                return Err(Error::Release);
            }
        }
        tx.execute("INSERT INTO trust_state VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET sequence=excluded.sequence,digest=excluded.digest", params![sequence,digest]).map_err(|_| Error::Storage)?;
        tx.commit().map_err(|_| Error::Storage)
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
}
