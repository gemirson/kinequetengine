//! KineSQL — Embedded storage engine with WAL, pages, and mmap.
//!
//! This crate implements [`kce_core::traits::StorageBackend`] for the KCE system.
//! It provides crash-safe persistence via a write-ahead log (WAL),
//! page-level checksums (CRC32), and memory-mapped reads.
//!
//! # Architecture
//!
//! - [`wal`] — Write-ahead log for crash recovery.
//! - [`page`] — Page management with CRC32 checksums.
//! - [`backend`] — [`KineSQL`] struct implementing [`kce_core::traits::StorageBackend`].
//!
//! # Example
//!
//! ```no_run
//! use kce_storage::backend::KineSQL;
//! use kce_core::traits::StorageBackend;
//!
//! let mut db = KineSQL::open("/tmp/kce_data.wal").expect("init");
//! db.write(b"key", b"value").expect("write");
//! db.flush().expect("fsync");
//! let val = db.read(b"key").expect("read");
//! assert_eq!(val, Some(b"value".to_vec()));
//! ```

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod backend;
#[allow(unsafe_code)]
pub mod io_uring_backend;
#[allow(unsafe_code)]
pub mod io_uring_splicer;
pub mod mmap;
pub mod page;
pub mod wal;

pub use backend::KineSQL;
