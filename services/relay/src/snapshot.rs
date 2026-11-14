use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Failures while publishing a SQLite snapshot.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SnapshotError {
    /// The generation the caller expected is no longer current.
    #[error("stale snapshot generation")]
    Stale,
    /// The snapshot upload failed after the local commit.
    #[error("snapshot upload failed")]
    UploadFailed,
    /// The snapshot file could not be read or written.
    #[error("snapshot io failed")]
    Io,
}

/// Durable copy of the SQLite file. `store` is the upload step.
pub trait SnapshotStore: Send + Sync {
    /// Current generation. Zero means no successful upload yet.
    fn generation(&self) -> u64;
    /// Bytes of the last successful snapshot.
    fn load(&self) -> Result<Option<Vec<u8>>, SnapshotError>;
    /// Upload `bytes` only when `expected` is the current generation.
    fn store(&self, bytes: &[u8], expected: u64) -> Result<u64, SnapshotError>;
}

/// Snapshot files on a local directory. Tests use this as the stand-in for Cloud Storage.
pub struct FileSnapshot {
    path: PathBuf,
    generation_path: PathBuf,
    fail_next: AtomicBool,
}

impl FileSnapshot {
    /// Create a snapshot store rooted at `path`.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let generation_path = path.with_extension("generation");
        Self {
            path,
            generation_path,
            fail_next: AtomicBool::new(false),
        }
    }

    /// The next `store` call fails before writing.
    pub fn fail_next_upload(&self) {
        self.fail_next.store(true, Ordering::SeqCst);
    }
}

impl SnapshotStore for FileSnapshot {
    fn generation(&self) -> u64 {
        fs::read_to_string(&self.generation_path)
            .ok()
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(0)
    }

    fn load(&self) -> Result<Option<Vec<u8>>, SnapshotError> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(SnapshotError::Io),
        }
    }

    fn store(&self, bytes: &[u8], expected: u64) -> Result<u64, SnapshotError> {
        if self.fail_next.swap(false, Ordering::SeqCst) {
            return Err(SnapshotError::UploadFailed);
        }
        let current = self.generation();
        if current != expected {
            return Err(SnapshotError::Stale);
        }
        fs::write(&self.path, bytes).map_err(|_| SnapshotError::Io)?;
        let next = current + 1;
        fs::write(&self.generation_path, next.to_string()).map_err(|_| SnapshotError::Io)?;
        Ok(next)
    }
}

/// When a store comes back stale, put the previous snapshot back on disk.
pub fn reload_previous(db_path: &Path, snapshot: &dyn SnapshotStore) -> Result<(), SnapshotError> {
    if let Some(previous) = snapshot.load()? {
        fs::write(db_path, previous).map_err(|_| SnapshotError::Io)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_generation_reloads_instead_of_overwriting() {
        let dir = std::env::temp_dir().join(format!("ouroboros-snap-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let db = dir.join("relay.sqlite");
        let snapshot = FileSnapshot::new(dir.join("relay.snapshot"));
        fs::write(&db, b"version-one").unwrap();
        snapshot.store(b"version-one", 0).unwrap();
        fs::write(&db, b"version-two").unwrap();
        let error = snapshot.store(b"version-two", 0).unwrap_err();
        assert_eq!(error, SnapshotError::Stale);
        reload_previous(&db, &snapshot).unwrap();
        assert_eq!(fs::read(&db).unwrap(), b"version-one");
        let _ = fs::remove_dir_all(&dir);
    }
}
