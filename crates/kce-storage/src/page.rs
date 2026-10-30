//! Page management and CRC32 checksums for the storage engine.
//!
//! Each [`Page`] holds a fixed-capacity data buffer protected by a CRC32
//! checksum.  Page size can be customised via [`PageConfig`].

use kce_core::caputo::CaputoState;
use kce_core::error::StorageError;

/// Default page size in bytes (8 KiB).
pub const DEFAULT_PAGE_SIZE: usize = 8192;

/// Fixed byte size of a CaputoState entry (48 bytes).
pub const CAPUTO_STATE_SIZE: usize = 48;

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

    /// FT-037: Appends a 48-byte fixed-size CaputoState directly to the page data buffer.
    ///
    /// Persisted zero-copy / lock-free without heap pointers for direct mmap reader access.
    pub fn append_caputo_state(&mut self, state: &CaputoState) -> Result<usize, StorageError> {
        let mut buf = [0u8; CAPUTO_STATE_SIZE];
        // 8 x f32 modes = 32 bytes
        for (i, mode) in state.modes.iter().enumerate() {
            buf[i * 4..(i + 1) * 4].copy_from_slice(&mode.to_le_bytes());
        }
        // last_updated_ms = 8 bytes
        buf[32..40].copy_from_slice(&state.last_updated_ms.to_le_bytes());
        // alpha = 4 bytes
        buf[40..44].copy_from_slice(&state.alpha.to_le_bytes());
        // cached_intensity = 4 bytes
        buf[44..48].copy_from_slice(&state.cached_intensity.to_le_bytes());

        let offset = self.data.len();
        self.data.extend_from_slice(&buf);
        self.checksum = crc32fast::hash(&self.data);
        Ok(offset)
    }

    /// FT-037: Reads a 48-byte fixed-size CaputoState from `offset` in page data.
    pub fn read_caputo_state(&self, offset: usize) -> Result<CaputoState, StorageError> {
        if offset + CAPUTO_STATE_SIZE > self.data.len() {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!(
                    "offset {} + 48 exceeds page data len {}",
                    offset,
                    self.data.len()
                ),
            )));
        }

        let slice = &self.data[offset..offset + CAPUTO_STATE_SIZE];
        let mut modes = [0.0_f32; 8];
        for i in 0..8 {
            let mut b = [0u8; 4];
            b.copy_from_slice(&slice[i * 4..(i + 1) * 4]);
            modes[i] = f32::from_le_bytes(b);
        }

        let mut b_u64 = [0u8; 8];
        b_u64.copy_from_slice(&slice[32..40]);
        let last_updated_ms = u64::from_le_bytes(b_u64);

        let mut b_alpha = [0u8; 4];
        b_alpha.copy_from_slice(&slice[40..44]);
        let alpha = f32::from_le_bytes(b_alpha);

        let mut b_cached = [0u8; 4];
        b_cached.copy_from_slice(&slice[44..48]);
        let cached_intensity = f32::from_le_bytes(b_cached);

        Ok(CaputoState {
            modes,
            last_updated_ms,
            alpha,
            cached_intensity,
        })
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

    #[test]
    fn test_caputo_state_page_roundtrip() {
        let mut page = Page::new(1, Vec::new());
        let original = CaputoState::new(0.55, 75.0, 12_345);

        let offset = page.append_caputo_state(&original).expect("append caputo");
        assert_eq!(offset, 0);
        assert_eq!(page.data.len(), CAPUTO_STATE_SIZE);
        assert!(page.verify().is_ok());

        let restored = page.read_caputo_state(offset).expect("read caputo");
        assert_eq!(original, restored);
    }
}
