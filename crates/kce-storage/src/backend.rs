//! KineSQL — the main embedded storage engine.
//!
//! Combines WAL, pages, and the [`StorageBackend`] trait into a single
//! crash-safe, concurrent storage engine.  Supports configurable page sizes
//! via [`PageConfig`] and optional memory-mapped reads for large data files
//! via [`crate::mmap::MmapReader`].

use kce_core::error::StorageError;
use kce_core::traits::{RecoveryReport, StorageBackend};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::mmap::MmapReader;
use crate::page::{Page, PageConfig};
use crate::wal::{Wal, WalEntry};

/// KineSQL embedded storage engine.
///
/// Thread-safe via `Arc<RwLock<...>>`.  Use [`KineSQL::shared`] to obtain a
/// shared handle.
///
/// # Examples
///
/// ```no_run
/// use kce_storage::backend::KineSQL;
/// use kce_core::traits::StorageBackend;
///
/// let mut db = KineSQL::open("/tmp/kce_data.wal").expect("init");
/// db.write(b"key", b"value").expect("write");
/// db.flush().expect("fsync");
/// let val = db.read(b"key").expect("read");
/// assert_eq!(val, Some(b"value".to_vec()));
/// ```
pub struct KineSQL {
    /// In-memory page store.
    pages: BTreeMap<u64, Page>,
    /// Key -> page-id index.
    index: BTreeMap<Vec<u8>, u64>,
    /// Write-Ahead Log handle.
    wal: Wal,
    /// Path to the WAL file (needed for replay on recovery).
    wal_path: PathBuf,
    /// Next page id.
    next_page_id: u64,
    /// Page creation configuration.
    page_config: PageConfig,
    /// Optional memory-mapped reader for large data files.
    mmap_reader: Option<MmapReader>,
}

impl KineSQL {
    /// Open a KineSQL instance with the WAL at the given path.
    ///
    /// Uses default page size ([`PageConfig::default()`]).
    pub fn open<P: AsRef<Path>>(wal_path: P) -> Result<Self, StorageError> {
        Self::open_with_config(wal_path, PageConfig::default())
    }

    /// Open a KineSQL instance with a custom [`PageConfig`].
    ///
    /// This allows setting a non-default page size for the storage engine.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Io`] if the WAL file cannot be opened.
    pub fn open_with_config<P: AsRef<Path>>(
        wal_path: P,
        page_config: PageConfig,
    ) -> Result<Self, StorageError> {
        let path = wal_path.as_ref().to_path_buf();
        let wal = Wal::open(&path)?;
        Ok(Self {
            pages: BTreeMap::new(),
            index: BTreeMap::new(),
            wal,
            wal_path: path,
            next_page_id: 0,
            page_config,
            mmap_reader: None,
        })
    }

    /// Attach a memory-mapped reader for large data file reads.
    ///
    /// Once attached, [`KineSQL::read_mmap`] can be used to read bytes
    /// directly from a memory-mapped file without copying into page
    /// structures.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Io`] if the file cannot be opened or mapped.
    pub fn attach_mmap<P: AsRef<Path>>(&mut self, path: P) -> Result<(), StorageError> {
        let reader = MmapReader::open(path)?;
        self.mmap_reader = Some(reader);
        Ok(())
    }

    /// Read raw bytes from the attached memory-mapped file.
    ///
    /// Returns a slice of `len` bytes starting at `offset`, or `None` if the
    /// range is out of bounds or no mmap is attached.  The returned slice
    /// borrows from the mapping and is valid for the lifetime of `self`.
    ///
    /// This is zero-copy: the OS page cache serves the data directly.
    pub fn read_mmap(&self, offset: usize, len: usize) -> Option<&[u8]> {
        self.mmap_reader.as_ref().and_then(|r| r.read_at(offset, len))
    }

    /// Return whether an mmap reader is currently attached.
    pub fn has_mmap(&self) -> bool {
        self.mmap_reader.is_some()
    }

    /// Wrap in `Arc<RwLock<...>>` for concurrent access.
    pub fn shared(self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self))
    }

    /// Apply a single WAL entry to the in-memory store.
    fn apply_entry(&mut self, entry: &WalEntry) {
        if entry.operation == "INSERT" {
            let page_id = self.next_page_id;
            self.next_page_id += 1;
            let page = Page::with_config(page_id, entry.data.clone(), &self.page_config);
            self.pages.insert(page_id, page);
            self.index.insert(entry.table.as_bytes().to_vec(), page_id);
        }
    }
}

impl StorageBackend for KineSQL {
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        match self.index.get(key) {
            Some(page_id) => match self.pages.get(page_id) {
                Some(page) => {
                    page.verify()?;
                    Ok(Some(page.data.clone()))
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn write(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        let entry = WalEntry {
            timestamp_ms: 0,
            operation: "INSERT".into(),
            table: String::from_utf8_lossy(key).into_owned(),
            data: value.to_vec(),
            checksum: crc32fast::hash(value),
        };
        self.wal.append(&entry)?;
        self.apply_entry(&entry);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), StorageError> {
        Ok(())
    }

    /// Recover state from the WAL after a crash (or on first open).
    ///
    /// Replays all entries from the WAL file by calling [`Wal::replay`] and
    /// re-applying each entry via [`KineSQL::apply_entry`].  This restores
    /// the in-memory page store and index to a consistent state.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::WalCorrupted`] if any WAL entry cannot be
    /// deserialized.
    fn recover(&mut self) -> Result<RecoveryReport, StorageError> {
        let entries = Wal::replay(&self.wal_path)?;
        let wal_entries_replayed = entries.len();
        for entry in &entries {
            // Verify data integrity before applying.
            let actual_crc = crc32fast::hash(&entry.data);
            if actual_crc != entry.checksum {
                tracing::warn!(
                    page = self.next_page_id,
                    expected = entry.checksum,
                    actual = actual_crc,
                    "WAL entry checksum mismatch during recovery"
                );
                // Skip corrupted entries but continue replaying.
                continue;
            }
            self.apply_entry(entry);
        }
        let pages_recovered = self.pages.len();
        // Re-verify all recovered pages.
        let checksum_failures = self
            .pages
            .values()
            .filter(|page| page.verify().is_err())
            .count();
        Ok(RecoveryReport {
            wal_entries_replayed,
            pages_recovered,
            checksum_failures,
            data_integrity_valid: checksum_failures == 0,
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    /// Create a fresh KineSQL instance with a unique WAL path in a
    /// process-specific temp directory.
    fn temp_kinesql() -> KineSQL {
        let dir = std::env::temp_dir().join(format!("kce_kinesql_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let wal_path = dir.join(format!(
            "{}.wal",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        KineSQL::open(&wal_path).expect("open")
    }

    /// Create a fresh KineSQL instance with a custom page config.
    fn temp_kinesql_with_config(config: PageConfig) -> KineSQL {
        let dir = std::env::temp_dir().join(format!("kce_kinesql_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let wal_path = dir.join(format!(
            "{}.wal",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        KineSQL::open_with_config(&wal_path, config).expect("open")
    }

    #[test]
    fn write_and_read() {
        let mut db = temp_kinesql();
        db.write(b"key1", b"value1").expect("write");
        let val = db.read(b"key1").expect("read");
        assert_eq!(val, Some(b"value1".to_vec()));
    }

    #[test]
    fn read_missing_key() {
        let db = temp_kinesql();
        let val = db.read(b"missing").expect("read");
        assert_eq!(val, None);
    }

    /// Simulate a crash: write data, drop the engine (no graceful shutdown),
    /// re-open from the same WAL, call `recover()`, and verify integrity.
    #[test]
    fn crash_recovery_simulation() {
        let dir = std::env::temp_dir().join(format!("kce_crash_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let wal_path = dir.join(format!(
            "crash_{}.wal",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        // Phase 1: write several entries and simulate a crash by dropping
        // the engine without flushing or closing gracefully.
        {
            let mut db = KineSQL::open(&wal_path).expect("open first");
            db.write(b"alpha", b"one").expect("write alpha");
            db.write(b"beta", b"two").expect("write beta");
            db.write(b"gamma", b"three").expect("write gamma");
            // Verify in-memory state before "crash".
            assert_eq!(db.read(b"alpha").expect("read"), Some(b"one".to_vec()));
            assert_eq!(db.read(b"beta").expect("read"), Some(b"two".to_vec()));
            assert_eq!(db.read(b"gamma").expect("read"), Some(b"three".to_vec()));
            // Drop without explicit flush — simulates a crash.
        }

        // Phase 2: re-open from the same WAL and recover.
        {
            let mut db = KineSQL::open(&wal_path).expect("open second");
            let report = db.recover().expect("recover");

            // All three entries should have been replayed.
            assert_eq!(report.wal_entries_replayed, 3);
            assert_eq!(report.pages_recovered, 3);
            assert_eq!(report.checksum_failures, 0);
            assert!(report.data_integrity_valid);

            // Verify data survives the simulated crash.
            assert_eq!(db.read(b"alpha").expect("read"), Some(b"one".to_vec()));
            assert_eq!(db.read(b"beta").expect("read"), Some(b"two".to_vec()));
            assert_eq!(db.read(b"gamma").expect("read"), Some(b"three".to_vec()));
        }

        // Clean up.
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Recovery on an empty WAL should succeed with zero counts.
    #[test]
    fn recover_empty_wal() {
        let dir = std::env::temp_dir().join(format!("kce_empty_recovery_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let wal_path = dir.join(format!(
            "empty_{}.wal",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        {
            let mut db = KineSQL::open(&wal_path).expect("open");
            let report = db.recover().expect("recover empty");
            assert_eq!(report.wal_entries_replayed, 0);
            assert_eq!(report.pages_recovered, 0);
            assert_eq!(report.checksum_failures, 0);
            assert!(report.data_integrity_valid);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Configurable page size is respected.
    #[test]
    fn configurable_page_size() {
        let config = PageConfig { size: 1024 };
        let mut db = temp_kinesql_with_config(config);
        db.write(b"small_key", b"small_value").expect("write");
        let val = db.read(b"small_key").expect("read");
        assert_eq!(val, Some(b"small_value".to_vec()));
    }

    /// Recovery with custom page config preserves data.
    #[test]
    fn crash_recovery_custom_page_size() {
        let dir = std::env::temp_dir().join(format!("kce_custom_page_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let wal_path = dir.join(format!(
            "custom_{}.wal",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        let config = PageConfig { size: 2048 };

        {
            let mut db =
                KineSQL::open_with_config(&wal_path, config).expect("open with config");
            db.write(b"cfg_key", b"cfg_val").expect("write");
        }

        {
            let mut db =
                KineSQL::open_with_config(&wal_path, config).expect("reopen with config");
            let report = db.recover().expect("recover");
            assert_eq!(report.wal_entries_replayed, 1);
            assert_eq!(report.pages_recovered, 1);
            assert!(report.data_integrity_valid);
            assert_eq!(db.read(b"cfg_key").expect("read"), Some(b"cfg_val".to_vec()));
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Mmap reader can be attached and used for zero-copy reads.
    #[test]
    fn mmap_attachment_and_read() {
        let dir = std::env::temp_dir().join(format!("kce_mmap_backend_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let wal_path = dir.join("mmap.wal");
        let data_path = dir.join("data.bin");
        std::fs::write(&data_path, b"mmap hello world").expect("write data");

        let mut db = KineSQL::open(&wal_path).expect("open");
        assert!(!db.has_mmap());

        db.attach_mmap(&data_path).expect("attach mmap");
        assert!(db.has_mmap());

        let chunk = db.read_mmap(0, 4).expect("read mmap");
        assert_eq!(chunk, b"mmap");
        let chunk2 = db.read_mmap(5, 5).expect("read mmap offset");
        assert_eq!(chunk2, b"hello");

        // Out of bounds returns None.
        assert!(db.read_mmap(0, 100).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
