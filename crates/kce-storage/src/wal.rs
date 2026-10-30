//! Write-Ahead Log (WAL) for crash-safe persistence.
//!
//! The WAL ensures that no data is lost on crash.  Writes are appended to the
//! log file and fsynced before being applied to the main data pages.

use kce_core::error::StorageError;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

/// A single entry in the WAL.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalEntry {
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Operation type (e.g. "INSERT", "UPDATE", "DELETE").
    pub operation: String,
    /// Target table / namespace.
    pub table: String,
    /// Serialized data payload.
    pub data: Vec<u8>,
    /// CRC32 checksum of the data field.
    pub checksum: u32,
}

/// Persistent Write-Ahead Log.
pub struct Wal {
    writer: BufWriter<File>,
}

impl Wal {
    /// Open (or create) a WAL file at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    /// Append an entry to the WAL and fsync.
    pub fn append(&mut self, entry: &WalEntry) -> Result<(), StorageError> {
        let bytes = serde_json::to_vec(entry).map_err(|e| {
            StorageError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;
        self.writer.write_all(&bytes)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;
        Ok(())
    }

    /// Replay all entries from a WAL file (for recovery).
    pub fn replay<P: AsRef<Path>>(path: P) -> Result<Vec<WalEntry>, StorageError> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();
        for (line_no, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let entry: WalEntry =
                serde_json::from_str(line).map_err(|e| StorageError::WalCorrupted {
                    offset: line_no as u64,
                    reason: e.to_string(),
                })?;
            entries.push(entry);
        }
        Ok(entries)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn wal_entry_serde_roundtrip() {
        let entry = WalEntry {
            timestamp_ms: 1000,
            operation: "INSERT".into(),
            table: "vectors".into(),
            data: b"hello".to_vec(),
            checksum: crc32fast::hash(b"hello"),
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let back: WalEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.operation, "INSERT");
        assert_eq!(back.checksum, entry.checksum);
    }

    #[test]
    fn wal_open_and_append() {
        let dir = std::env::temp_dir().join("kce_wal_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("test.wal");

        let mut wal = Wal::open(&path).expect("open wal");
        let entry = WalEntry {
            timestamp_ms: 1000,
            operation: "INSERT".into(),
            table: "test".into(),
            data: b"data".to_vec(),
            checksum: crc32fast::hash(b"data"),
        };
        wal.append(&entry).expect("append");

        let entries = Wal::replay(&path).expect("replay");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].table, "test");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
