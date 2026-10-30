//! FT-033 — io_uring Zero-Copy Splice.
//!
//! Kernel-level data routing between files and sockets with zero-copy bypass.

use std::os::unix::io::RawFd;
use std::sync::Arc;
use io_uring::{opcode, squeue, types, IoUring};
use parking_lot::Mutex;

use kce_core::error::StorageError;

/// Manager for Zero-Copy Splice operations.
pub struct IoUringSplicer {
    ring: Arc<Mutex<IoUring>>,
    /// Kernel pipe FDs used as intermediate buffers for splice.
    pipe: (RawFd, RawFd),
}

impl IoUringSplicer {
    /// Create a new splicer using an existing io_uring handle.
    pub fn new(ring: Arc<Mutex<IoUring>>) -> Result<Self, StorageError> {
        let mut fds = [0; 2];
        unsafe {
            if libc::pipe(fds.as_mut_ptr()) < 0 {
                return Err(StorageError::Io(std::io::Error::last_os_error()));
            }
            // Set large pipe capacity to minimize context switches (FT-033 AC-013)
            libc::fcntl(fds[0], libc::F_SETPIPE_SZ, 1024 * 1024);
        }

        Ok(Self {
            ring,
            pipe: (fds[0], fds[1]),
        })
    }

    /// Submit a Zero-Copy Splice from File to Socket (FT-033 AC-011).
    pub fn submit_splice(
        &self,
        fd_in: RawFd,
        off_in: i64,
        fd_out: RawFd,
        len: u32,
        user_data: u64,
    ) -> Result<(), StorageError> {
        let (p_read, p_write) = self.pipe;

        // 1. Splice from Source File to Kernel Pipe (Input)
        let op1 = opcode::Splice::new(
            types::Fd(fd_in),
            off_in,
            types::Fd(p_write),
            -1,
            len
        )
        .build()
        .flags(squeue::Flags::IO_LINK) // Link to the next step
        .user_data(user_data);

        // 2. Splice from Kernel Pipe to Destination Socket (Output)
        let op2 = opcode::Splice::new(
            types::Fd(p_read),
            -1,
            types::Fd(fd_out),
            -1,
            len
        )
        .build()
        .user_data(user_data + 1);

        let mut ring = self.ring.lock();
        unsafe {
            let mut sq = ring.submission();
            sq.push(&op1).map_err(|_| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, "SQ full")))?;
            sq.push(&op2).map_err(|_| StorageError::Io(std::io::Error::new(std::io::ErrorKind::Other, "SQ full")))?;
        }

        ring.submit().map_err(|e| StorageError::Io(e))?;
        Ok(())
    }
}

impl Drop for IoUringSplicer {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.pipe.0);
            libc::close(self.pipe.1);
        }
    }
}
