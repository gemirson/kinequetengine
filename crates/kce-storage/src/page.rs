//! Page management and CRC32 checksums for the storage engine.
//!
//! Each [`Page`] holds a fixed-capacity data buffer protected by a CRC32
//! checksum.  Page size can be customised via [`PageConfig`].

use kce_core::error::StorageError;

/// Default page size in bytes (8 KiB).
pub const DEFAULT_PAGE_SIZE: usize = 8192;

/// Configuration for page creation.
///
/// Controls the maximum data capacity per page.  Pass a `PageConfig` to
/// [`Page::with_config`] when the default 8 KiB page size is not suitable.
///
/// # Examples
///
/// ```
/// use kce_storage::page::{Page, PageConfig};
///
/// let config = PageConfig { size: 4096 };
/// let page = Page::with_config(0, b"short data".to_vec(), &config);
/// assert_eq!(page.capacity(), 4096);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PageConfig {
    /// Maximum page data size in bytes.
    pub size: usize,
}

impl Default for PageConfig {
    fn default() -> Self {
        Self {
            size: DEFAULT_PAGE_SIZE,
        }
    }
}

/// A storage page with its data and checksum.
#[derive(Debug, Clone)]
pub struct Page {
    /// Page identifier.
    pub id: u64,
    /// Raw data bytes.
    pub data: Vec<u8>,
    /// CRC32 checksum of `data`.
    pub checksum: u32,
    /// Maximum capacity in bytes (informational).
    capacity: usize,
}

impl Page {
    /// Create a new page using the default page size ([`DEFAULT_PAGE_SIZE`]).
    ///
    /// Computes and stores the CRC32 checksum of `data`.
    pub fn new(id: u64, data: Vec<u8>) -> Self {
        Self::with_config(id, data, &PageConfig::default())
    }

    /// Create a new page with a custom [`PageConfig`].
    ///
    /// The `data` vector may be shorter than `config.size`; the capacity is
    /// recorded for informational purposes (e.g. deciding when to split).
    /// If `data` exceeds the configured size, it is still accepted (no
    /// truncation) so that callers are not forced to split before writing.
    pub fn with_config(id: u64, data: Vec<u8>, config: &PageConfig) -> Self {
        let checksum = crc32fast::hash(&data);
        Self {
            id,
            data,
            checksum,
            capacity: config.size,
        }
    }

    /// Verify the checksum.  Returns `Ok(())` if valid, `Err` otherwise.
    pub fn verify(&self) -> Result<(), StorageError> {
        let actual = crc32fast::hash(&self.data);
        if actual == self.checksum {
            Ok(())
        } else {
            Err(StorageError::ChecksumFailure {
                page_id: self.id,
                expected: self.checksum,
                actual,
            })
        }
    }

    /// Return the page's configured maximum capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Compute a CRC32 checksum over arbitrary bytes.
pub fn checksum(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn page_checksum_valid() {
        let page = Page::new(1, b"hello world".to_vec());
        assert!(page.verify().is_ok());
    }

    #[test]
    fn page_checksum_corrupted() {
        let mut page = Page::new(1, b"hello world".to_vec());
        page.data[0] = b'x';
        let result = page.verify();
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::ChecksumFailure { page_id, .. } => assert_eq!(page_id, 1),
            other => panic!("expected ChecksumFailure, got {:?}", other),
        }
    }

    #[test]
    fn checksum_deterministic() {
        let a = checksum(b"test data");
        let b = checksum(b"test data");
        assert_eq!(a, b);
    }

    #[test]
    fn checksum_different_data() {
        let a = checksum(b"data a");
        let b = checksum(b"data b");
        assert_ne!(a, b);
    }

    #[test]
    fn page_default_capacity() {
        let page = Page::new(0, b"test".to_vec());
        assert_eq!(page.capacity(), DEFAULT_PAGE_SIZE);
    }

    #[test]
    fn page_custom_capacity() {
        let config = PageConfig { size: 4096 };
        let page = Page::with_config(0, b"test".to_vec(), &config);
        assert_eq!(page.capacity(), 4096);
        assert!(page.verify().is_ok());
    }

    #[test]
    fn page_config_default() {
        let config = PageConfig::default();
        assert_eq!(config.size, DEFAULT_PAGE_SIZE);
    }
}
