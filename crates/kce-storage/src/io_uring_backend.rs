//! FT-032 — io_uring Bare-Metal Storage I/O.
//!
//! Provides a high-performance, asynchronous I/O backend for KineSQL using Linux io_uring.

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use io_uring::{opcode, squeue, types, IoUring};
use parking_lot::Mutex;

use kce_core::error::StorageError;

/// Backend for high-performance storage I/O.
pub struct IoUringBackend {
    ring: Arc<Mutex<IoUring>>,
    #[allow(dead_code)]
    sq_poll: bool,
}

impl IoUringBackend {
    /// Initialize a new io_uring backend.
    ///
    /// Fallback mechanism (AC-014) is handled by the caller or during open.
    pub fn new(entries: u32, sq_poll: bool) -> Result<Self, StorageError> {
        let mut builder = IoUring::builder();
        if sq_poll {
            builder.setup_sqpoll(2000); // 2ms idle timeout for kernel thread
        }

        let ring = builder.build(entries).map_err(|e| StorageError::Io(e))?;
        
        Ok(Self {
            ring: Arc::new(Mutex::new(ring)),
            sq_poll,
        })
    }

    /// Submit an asynchronous write operation (FT-032 AC-011).
    pub fn submit_write(
        &self,
        file: &File,
        offset: u64,
        data: *const u8,
        len: u32,
        user_data: u64,
    ) -> Result<(), StorageError> {
        let fd = types::Fd(file.as_raw_fd());
        let write_op = opcode::Write::new(fd, data, len)
            .offset(offset)
            .build()
            .user_data(user_data);

        let mut ring = self.ring.lock();
        unsafe {
            ring.submission()
                .push(&write_op)
                .map_err(|_| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, "SQ full")))?;
        }
        
        ring.submit().map_err(|e| StorageError::Io(e))?;
        Ok(())
    }

    /// Submit a linked write + fsync operation (FT-032 AC-012).
    pub fn submit_atomic_write(
        &self,
        file: &File,
        offset: u64,
        data: *const u8,
        len: u32,
        user_data: u64,
    ) -> Result<(), StorageError> {
        let fd = types::Fd(file.as_raw_fd());
        
        // 1. Write op with IO_LINK
        let write_op = opcode::Write::new(fd, data, len)
            .offset(offset)
            .build()
            .flags(squeue::Flags::IO_LINK)
            .user_data(user_data);

        // 2. Fsync op
        let fsync_op = opcode::Fsync::new(fd)
            .build()
            .user_data(user_data + 1);

        let mut ring = self.ring.lock();
        unsafe {
            let mut sq = ring.submission();
            sq.push(&write_op).map_err(|_| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, "SQ full")))?;
            sq.push(&fsync_op).map_err(|_| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, "SQ full")))?;
        }
        
        ring.submit().map_err(|e| StorageError::Io(e))?;
        Ok(())
    }

    /// Harvest completed operations from the CQ.
    pub fn poll_completions(&self) -> Vec<u64> {
        let mut ring = self.ring.lock();
        let mut completed = Vec::new();
        
        while let Some(cqe) = ring.completion().next() {
            if cqe.result() >= 0 {
                completed.push(cqe.user_data());
            } else {
                tracing::error!(
                    error_code = cqe.result(),
                    user_data = cqe.user_data(),
                    "io_uring operation failed"
                );
            }
        }
        completed
    }
}
