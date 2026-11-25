//! The Windows transport of the UI link (#3874): a named pipe with a
//! security descriptor that admits the window's own user only
//! (`link_pipe_acl`), so the kernel - not a token on the wire - decides
//! who the peer is. The pipe name carries the window's process id, and
//! the CLI asks the kernel which process serves the pipe before it
//! sends the token: a TCP port cannot be verified this way, because a
//! peer that never read the discovery file still receives the token
//! and can answer anything.
//!
//! Every wait is bounded and every handle is closed on its owner's
//! drop, never by a panic. The byte stream itself is `link_pipe_stream`.
//!
//! Provenance: our own FFI over the platform's published named-pipe
//! API (Microsoft Learn: CreateNamedPipeW, ConnectNamedPipe,
//! GetNamedPipeServerProcessId). Clean-room, public sources only.

#![allow(clippy::undocumented_unsafe_blocks)] // every block names its API below

use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_IO_PENDING, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, GENERIC_READ,
    GENERIC_WRITE, GetLastError, HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT,
};
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, FILE_FLAG_OVERLAPPED, OPEN_EXISTING};
use windows_sys::Win32::System::IO::{CancelIoEx, GetOverlappedResultEx, OVERLAPPED};
use windows_sys::Win32::System::Pipes::{ConnectNamedPipe, GetNamedPipeServerProcessId};

use super::link_pipe_acl::{SecurityDescriptor, create_instance};
// The stream type is this transport's public face: `wire` and the
// tests name it through this module.
pub use super::link_pipe_stream::PipeStream;
use super::link_pipe_stream::OpEvent;

/// How long one slice of the connect wait runs before the stop flag is
/// looked at again: the bound on how long dropping the listener takes.
const WAIT_SLICE: u32 = 50;

/// The pipe name of the window with this process id: the discovery
/// file's `address` field on Windows. Derived from the pid, so the CLI
/// can refuse a file whose pipe name belongs to another pid.
#[must_use]
pub fn pipe_name(pid: u32) -> String {
    format!(r"\\.\pipe\camcraft-ui-link-{pid}")
}

fn wide(text: &str) -> Vec<u16> {
    OsStr::new(text).encode_wide().chain([0]).collect()
}

pub(super) fn close(handle: HANDLE) {
    // Safety: the caller owns the handle; no other thread uses it.
    unsafe {
        if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
            CloseHandle(handle);
        }
    }
}

/// A raw pipe handle on its way between threads: the creator thread,
/// the accept channel, the CLI's bridge threads. One owner at a time,
/// so the move is sound even though a raw pointer is not `Send` alone.
#[derive(Clone, Copy)]
struct RawHandle(HANDLE);
// Safety: a kernel handle is thread-agnostic; each move hands the only
// user of this end of the pipe to the next thread.
unsafe impl Send for RawHandle {}

enum Connect {
    /// A client took the instance.
    Arrived,
    /// The listener is gone: end without a client.
    Cancelled,
    /// The OS refused the wait; the caller makes a fresh instance.
    Failed,
}

/// Wait for a client in short slices, so the stop flag is seen even
/// while the wait is pending, and a cancel settles it at once.
///
/// # Safety
/// `handle` is a listening pipe instance that no other thread uses.
/// Wait for a client in short slices, so the stop flag is seen even
/// while the wait is pending, and a cancel settles it at once.
///
/// # Errors
/// The operation's event could not be made.
///
/// # Safety
/// `handle` is a listening pipe instance that no other thread uses.
unsafe fn wait_connect(handle: HANDLE, stop: &AtomicBool) -> std::io::Result<Connect> {
    // The connect waits on its own event, like every other operation:
    // a handle wait could see a stale signal of the instance object.
    let event = OpEvent::new()?;
    let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
    overlapped.hEvent = event.handle();
    if unsafe { ConnectNamedPipe(handle, &mut overlapped) } != 0 {
        return Ok(Connect::Arrived);
    }
    let pending = unsafe { GetLastError() };
    if pending == ERROR_PIPE_CONNECTED {
        return Ok(Connect::Arrived);
    }
    if pending != ERROR_IO_PENDING {
        return Ok(Connect::Failed);
    }
    loop {
        let mut taken = 0_u32;
        if unsafe { GetOverlappedResultEx(handle, &overlapped, &mut taken, WAIT_SLICE, 0) } != 0 {
            return Ok(Connect::Arrived);
        }
        let err = unsafe { GetLastError() };
        if err == WAIT_TIMEOUT {
            if stop.load(Ordering::Acquire) {
                unsafe { CancelIoEx(handle, std::ptr::null()) };
                return Ok(Connect::Cancelled);
            }
            continue;
        }
        // A client that connected and left before the wait: retry.
        return Ok(Connect::Failed);
    }
}

struct ListenerState {
    stop: AtomicBool,
    /// The instance whose connect wait is pending (or was just made):
    /// Drop cancels it, and the thread closes it when it sees `stop`.
    pending: Mutex<Option<RawHandle>>,
}

/// The window's end of the pipe: a thread that keeps one instance
/// waiting for the next client, and hands each connected instance to
/// [`accept`](Self::accept), which never blocks.
pub struct PipeListener {
    connected: Receiver<io::Result<PipeStream>>,
    state: Arc<ListenerState>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl PipeListener {
    /// Serve the pipe `name` (fail fast: the first instance exists, or
    /// the name is taken and the error says so).
    ///
    /// # Errors
    /// The security descriptor or the pipe creation failed.
    pub fn new(name: &str) -> io::Result<Self> {
        let wide = wide(name);
        let state = Arc::new(ListenerState {
            stop: AtomicBool::new(false),
            pending: Mutex::new(None),
        });
        // The first instance exists before the link is said to be up:
        // a taken name (a squatter) is an error now. The kernel copies
        // the descriptor into the pipe, so it is freed right after.
        let first = {
            let sd = SecurityDescriptor::user_only()?;
            create_instance(&wide, sd.attributes(), true)?
        };
        *state.pending.lock().expect("the listener lock") = Some(RawHandle(first));
        // The thread's own copy: a raw handle crosses as `RawHandle`.
        let first = RawHandle(first);
        let (tx, connected) = mpsc::channel();
        let worker = Arc::clone(&state);
        let thread = std::thread::Builder::new()
            .name("camcraft-ui-link-pipe".into())
            .spawn(move || {
                // Each later instance gets its own descriptor: one
                // built here, never a captured pointer.
                let sd = match SecurityDescriptor::user_only() {
                    Ok(sd) => sd,
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        return;
                    }
                };
                // The whole `RawHandle` crosses (a bare field capture
                // would move the raw pointer itself, which is not Send).
                let mut handle = first;
                loop {
                    if worker.stop.load(Ordering::Acquire) {
                        close(handle.0);
                        break;
                    }
                    let connect = match unsafe { wait_connect(handle.0, &worker.stop) } {
                        Ok(c) => c,
                        Err(e) => {
                            close(handle.0);
                            let _ = tx.send(Err(e));
                            break;
                        }
                    };
                    match connect {
                        Connect::Arrived => {
                            *worker.pending.lock().expect("the listener lock") = None;
                            if worker.stop.load(Ordering::Acquire) {
                                close(handle.0);
                                break;
                            }
                            // The stream owns the instance from here:
                            // the receiver drops it, or this send does
                            // - a connected instance is never left open,
                            // or the pipe's name could never free.
                            if tx.send(Ok(PipeStream::from_handle(handle.0))).is_err() {
                                break;
                            }
                        }
                        Connect::Cancelled => {
                            close(handle.0);
                            break;
                        }
                        Connect::Failed => {
                            close(handle.0);
                            if worker.stop.load(Ordering::Acquire) {
                                break;
                            }
                        }
                    }
                    // The next instance, so the next client connects
                    // while this one is still served (the backlog).
                    handle = match create_instance(&wide, sd.attributes(), false) {
                        Ok(h) => RawHandle(h),
                        Err(e) => {
                            let _ = tx.send(Err(e));
                            break;
                        }
                    };
                    *worker.pending.lock().expect("the listener lock") = Some(handle);
                }
            })?;
        Ok(Self {
            connected,
            state,
            thread: Some(thread),
        })
    }

    /// Take the next connected client, never blocking.
    ///
    /// # Errors
    /// `WouldBlock` while no client waits; the creation error, once,
    /// when the pipe could not make its next instance.
    pub fn accept(&self) -> io::Result<(PipeStream, String)> {
        match self.connected.try_recv() {
            Ok(Ok(stream)) => Ok((stream, "named pipe".to_owned())),
            Ok(Err(e)) => Err(e),
            // The creator thread ended (its error was delivered once):
            // report an empty listener, not a new error on every poll.
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "no pipe client waits",
            )),
        }
    }

    /// The accept loop polls; the pipe wait is sliced by itself.
    pub fn set_nonblocking(&self, _on: bool) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for PipeListener {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
        // Wake the thread out of a pending connect wait; a thread that
        // has not started its wait sees `stop` at its next slice.
        let pending = self.state.pending.lock().expect("the listener lock").take();
        if let Some(handle) = pending {
            unsafe { CancelIoEx(handle.0, std::ptr::null()) };
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Open the client end of the pipe `name`.
///
/// # Errors
/// `TimedOut` when every instance is busy (the window's one session is
/// taken); the OS error otherwise.
pub fn connect(name: &str) -> io::Result<PipeStream> {
    let wide = wide(name);
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        let why = unsafe { GetLastError() };
        if why == ERROR_PIPE_BUSY {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the window's link is busy: another client holds the session",
            ));
        }
        return Err(io::Error::from_raw_os_error(why as i32));
    }
    Ok(PipeStream::from_handle(handle))
}

/// The process id of the pipe's server side, as the kernel reports it.
/// The CLI's proof that the peer is the process the link file names
/// (#3874): it runs before one byte of the token crosses.
///
/// # Errors
/// The kernel could not say (the handle is not a pipe).
pub fn server_pid(stream: &PipeStream) -> io::Result<u32> {
    let mut pid = 0_u32;
    if unsafe { GetNamedPipeServerProcessId(stream.handle(), &mut pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(pid)
}
