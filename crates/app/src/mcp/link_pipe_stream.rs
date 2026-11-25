//! The byte stream of the UI link's Windows pipe (#3874): one
//! connected end, read and written through overlapped I/O so every
//! wait carries its own deadline - the synchronous pipe API has none,
//! and the idle law of `listen.rs` and the hello deadline need one.
//! Read and write deadlines are shared by every clone of one pipe, as
//! a socket option is.
//!
//! Provenance: our own FFI over the platform's published pipe API
//! (Microsoft Learn: ReadFile, WriteFile, GetOverlappedResultEx,
//! CancelIoEx). Clean-room, public sources only.

#![allow(clippy::undocumented_unsafe_blocks)] // every block names its API below

use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows_sys::Win32::Foundation::{
    DUPLICATE_SAME_ACCESS, DuplicateHandle, ERROR_BROKEN_PIPE, ERROR_IO_PENDING,
    ERROR_NO_DATA, ERROR_OPERATION_ABORTED, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
    WAIT_TIMEOUT,
};
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows_sys::Win32::System::IO::{CancelIoEx, GetOverlappedResultEx, OVERLAPPED};
use windows_sys::Win32::System::Threading::{CreateEventW, GetCurrentProcess};

use super::link_pipe::close;

/// The event of one overlapped operation: each op waits on its own
/// event, never on the pipe handle - completions signal the shared
/// instance object, so a handle wait cannot tell one completion from
/// another (the server's connect, this side's earlier write).
pub(super) struct OpEvent(HANDLE);

impl OpEvent {
    /// A fresh unnamed auto-reset event.
    ///
    /// # Errors
    /// The OS could not make the event.
    pub(super) fn new() -> std::io::Result<Self> {
        let made = unsafe { CreateEventW(std::ptr::null(), 0, 0, std::ptr::null()) };
        if made.is_null() || made == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self(made))
    }

    /// The raw event handle, set into an operation's OVERLAPPED.
    pub(super) fn handle(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OpEvent {
    fn drop(&mut self) {
        close(self.0);
    }
}

/// Read and write deadlines, shared by every clone of one pipe.
#[derive(Default)]
struct StreamLimits(Mutex<(Option<Duration>, Option<Duration>)>);

/// One connected end of a pipe: the window's accepted client, or the
/// CLI's connection to the window.
pub struct PipeStream {
    handle: HANDLE,
    limits: Arc<StreamLimits>,
}

impl PipeStream {
    pub(super) fn from_handle(handle: HANDLE) -> Self {
        Self {
            handle,
            limits: Arc::new(StreamLimits(Mutex::new((None, None)))),
        }
    }

    /// The raw handle, for the kernel's own queries about this pipe.
    pub(super) fn handle(&self) -> HANDLE {
        self.handle
    }

    fn ms(limits: &StreamLimits, write: bool) -> u32 {
        let guard = limits.0.lock().expect("the stream lock");
        let set = if write { guard.1 } else { guard.0 };
        // None: wait for ever, as a socket with no timeout does.
        set.map_or(u32::MAX, |d| d.as_millis().min(u32::MAX as u128) as u32)
    }

    /// # Errors
    /// A zero duration, like a socket timeout of zero.
    pub fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        if t.is_some_and(|d| d.is_zero()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a pipe read timeout cannot be zero",
            ));
        }
        self.limits.0.lock().expect("the stream lock").0 = t;
        Ok(())
    }

    /// # Errors
    /// A zero duration, like a socket timeout of zero.
    pub fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        if t.is_some_and(|d| d.is_zero()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a pipe write timeout cannot be zero",
            ));
        }
        self.limits.0.lock().expect("the stream lock").1 = t;
        Ok(())
    }

    /// The pipe API has no non-blocking mode that our callers need:
    /// every wait already runs under a deadline.
    pub fn set_nonblocking(&self, _on: bool) -> io::Result<()> {
        Ok(())
    }

    /// A second handle on the same pipe (one reads, one writes).
    ///
    /// # Errors
    /// The OS could not duplicate the handle.
    pub fn try_clone(&self) -> io::Result<Self> {
        let mut twin: HANDLE = std::ptr::null_mut();
        let made = unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                self.handle,
                GetCurrentProcess(),
                &mut twin,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        };
        if made == 0 || twin.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            handle: twin,
            limits: Arc::clone(&self.limits),
        })
    }

    /// A pipe cannot close one direction only. The caller's session
    /// still ends cleanly: the idle drop on the window's side, or the
    /// broken pipe when this process exits, closes it.
    pub fn shutdown_write(&self) -> io::Result<()> {
        Ok(())
    }

    /// One overlapped read under a deadline.
    fn read_into(&self, buf: &mut [u8]) -> io::Result<usize> {
        // Safety: the buffer is valid and only this call uses it.
        unsafe { self.io(false, buf.as_mut_ptr(), buf.len()) }
    }

    /// One overlapped write under a deadline. The pointer is cast mut
    /// for the shared core only; the OS reads from it, never writes.
    fn write_from(&self, buf: &[u8]) -> io::Result<usize> {
        // Safety: the buffer is valid for the length, and the OS only
        // reads through it on the write path.
        unsafe { self.io(true, buf.as_ptr() as *mut u8, buf.len()) }
    }

    /// One overlapped file operation under a deadline in milliseconds:
    /// on timeout the operation is cancelled and reaped, so the handle
    /// never keeps a pending operation past the caller's bounds.
    ///
    /// # Safety
    /// `buf` is valid for `len` bytes; only this call uses the handle.
    unsafe fn io(&self, write: bool, buf: *mut u8, len: usize) -> io::Result<usize> {
        // Each operation carries its own event: the wait below must
        // see THIS operation's completion, not the instance's last one.
        let event = OpEvent::new()?;
        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = event.handle();
        let mut moved = 0_u32;
        let said = unsafe {
            if write {
                WriteFile(self.handle, buf, len as u32, &mut moved, &mut overlapped)
            } else {
                ReadFile(self.handle, buf, len as u32, &mut moved, &mut overlapped)
            }
        };
        if said != 0 {
            return Ok(moved as usize);
        }
        let pending = unsafe { GetLastError() };
        if pending != ERROR_IO_PENDING {
            return self.finished(write, pending);
        }
        let wait = Self::ms(&self.limits, write);
        if unsafe { GetOverlappedResultEx(self.handle, &overlapped, &mut moved, wait, 0) } != 0 {
            return Ok(moved as usize);
        }
        let why = unsafe { GetLastError() };
        if why == WAIT_TIMEOUT {
            unsafe { CancelIoEx(self.handle, std::ptr::null()) };
            // Reap the settled operation: data that raced the deadline
            // still counts, a true timeout does not.
            if unsafe { GetOverlappedResultEx(self.handle, &overlapped, &mut moved, u32::MAX, 0) }
                != 0
            {
                return Ok(moved as usize);
            }
            return self.finished(write, unsafe { GetLastError() });
        }
        self.finished(write, why)
    }

    /// The error (or the end of input) of a completed operation.
    fn finished(&self, write: bool, code: u32) -> io::Result<usize> {
        match code {
            // Reading a pipe whose writer hung up is the end of input.
            ERROR_BROKEN_PIPE | ERROR_NO_DATA if !write => Ok(0),
            ERROR_OPERATION_ABORTED => Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "the pipe operation timed out",
            )),
            _ => Err(io::Error::from_raw_os_error(code as i32)),
        }
    }
}

impl Drop for PipeStream {
    fn drop(&mut self) {
        close(self.handle);
    }
}

impl std::io::Read for PipeStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.read_into(buf)
    }
}

impl std::io::Write for PipeStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_from(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(()) // byte pipes hand each written buffer to the peer at once
    }
}

// Safety: a kernel handle is thread-agnostic; a clone is a separate
// handle of its own, used by one thread at a time.
unsafe impl Send for PipeStream {}
