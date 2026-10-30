//! Memory-mapped file reader for large data files.
//!
//! Provides read-only access to files via OS-level memory mapping,
//! avoiding explicit read syscalls for large sequential or random-access
//! workloads.  The single `unsafe` block lives in [`MmapReader::open`]
//! and is wrapped behind a safe public API.

use kce_core::error::StorageError;
use std::fs::File;
use std::path::Path;

/// Read-only memory-mapped file reader.
///
/// Wraps [`memmap2::Mmap`] behind a safe interface.  The underlying file is
/// opened read-only, so concurrent writes are rejected by the OS.
///
/// # Examples
///
/// ```no_run
/// use kce_storage::mmap::MmapReader;
///
/// let reader = MmapReader::open("/tmp/large_data.bin").expect("open");
/// if let Some(chunk) = reader.read_at(0, 64) {
///     // process first 64 bytes
///     assert!(chunk.len() <= 64);
/// }
/// ```
pub struct MmapReader {
    mmap: memmap2::Mmap,
    size: usize,
}

impl MmapReader {
    /// Open a file and create a read-only memory mapping.
    ///
    /// The file is opened with read-only permissions.  The mapping reflects
    /// the file contents at the time of the call; subsequent modifications to
    /// the file are **not** guaranteed to be visible.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Io`] if the file cannot be opened or the
    /// operating system refuses to create the mapping.
    #[allow(unsafe_code)]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let file = File::open(path)?;
        let size = file.metadata()?.len() as usize;
        // SAFETY: The file descriptor is read-only.  We never modify the file
        // while the mapping is alive, and we do not expose a mutable reference
        // to the mapped memory.  The OS guarantees the mapped region reflects
        // the file at the time of mapping.
        let mmap = unsafe { memmap2::Mmap::map(&file) }
            .map_err(|e| StorageError::Io(std::io::Error::other(e)))?;
        Ok(Self { mmap, size })
    }

    /// Read a slice of `len` bytes starting at `offset`.
    ///
    /// Returns `None` if the requested range exceeds the mapped region.
    pub fn read_at(&self, offset: usize, len: usize) -> Option<&[u8]> {
        let end = offset.checked_add(len)?;
        self.mmap.get(offset..end)
    }

    /// Return the total size of the mapped file in bytes.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Return whether the mapped file is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Return a reference to the full mapped region as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn mmap_reader_open_and_read() {
        let dir = std::env::temp_dir().join(format!("kce_mmap_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("test_data.bin");
        std::fs::write(&path, b"hello world").expect("write");

        let reader = MmapReader::open(&path).expect("open");
        assert_eq!(reader.len(), 11);
        assert!(!reader.is_empty());
        assert_eq!(reader.read_at(0, 5), Some(&b"hello"[..]));
        assert_eq!(reader.read_at(6, 5), Some(&b"world"[..]));
        assert_eq!(reader.read_at(0, 100), None);
        assert_eq!(reader.as_slice(), b"hello world");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mmap_reader_empty_file() {
        let dir = std::env::temp_dir().join(format!("kce_mmap_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("empty.bin");
        std::fs::write(&path, b"").expect("write");

        let reader = MmapReader::open(&path).expect("open");
        assert_eq!(reader.len(), 0);
        assert!(reader.is_empty());
        assert_eq!(reader.read_at(0, 1), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mmap_reader_offset_bounds() {
        let dir = std::env::temp_dir().join(format!("kce_mmap_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("bounds.bin");
        std::fs::write(&path, b"abcdef").expect("write");

        let reader = MmapReader::open(&path).expect("open");
        assert_eq!(reader.read_at(3, 3), Some(&b"def"[..]));
        assert_eq!(reader.read_at(6, 1), None);
        assert_eq!(reader.read_at(100, 1), None);
        // Overflow protection
        assert_eq!(reader.read_at(usize::MAX, 1), None);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
