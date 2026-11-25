//! The CLI's trust law of the UI link (#3874): who the peer is, and how
//! a line from it is read. The transport proves the peer before one
//! byte of the token crosses - the 0700 socket folder on Unix, the
//! pipe's user-only ACL and the kernel-reported server process id on
//! Windows - and a TCP address is refused outright, because a token
//! sent to an open port is given away to whoever holds it. After the
//! peer is proven, every answer line is read under a byte cap and the
//! hello exchange under one elapsed deadline: a peer that trickles
//! bytes ends on the same clock as a silent one.
//!
//! Provenance: our own law over our own transports (link.rs,
//! link_pipe.rs); the framing is the MCP stdio transport
//! (modelcontextprotocol.io, revision 2025-06-18). Clean-room, public
//! sources only.

use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use super::link::{self, Entry};
use super::wire::Stream;
use crate::clock::Instant;

/// How long the whole hello exchange may take - the connect, the hello
/// line and the window's answer - before a window is read as busy or
/// too slow (a stale file, a hung window, a peer that trickles bytes:
/// one elapsed deadline ends all of them).
pub(super) const HELLO_TIMEOUT: Duration = Duration::from_secs(5);

/// The most bytes of one answer line (#3874): a peer that sends one
/// line with no end cannot make the client use all memory, and a line
/// that reaches the bound without its newline ends the link - a
/// remainder of it is never read as a new message. A tool result never
/// approaches the bound. After the hello, no read deadline applies (a
/// tool may run for minutes, #3751) - the peer proved its transport,
/// so the byte bound is what remains to check.
pub(super) const MAX_ANSWER: u64 = 1024 * 1024;

pub(super) fn timed_out(what: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::TimedOut, what)
}

pub(super) fn overlong(cap: u64) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!(
            "a link line passed the {cap} byte bound without its end - the connection is closed"
        ),
    )
}

/// Open the entry's transport only. This is where the peer proves what
/// it is, before one byte of the token crosses (#3874):
/// - `"tcp"` is refused outright: any local process can hold an open
///   port, and a token sent to it is given away - no exchange after
///   that can tell the real window from the holder.
/// - `"pipe"` (Windows) must be the entry pid's own pipe name, and the
///   kernel must report the pipe's server as that same process.
/// - `"unix"` (Unix) lives in the user's 0700 link folder: only the
///   user's own processes can hold the path.
///
/// # Errors
/// The transport is refused by name or by peer, or the OS connect
/// failed.
pub(super) fn dial(entry: &Entry) -> std::io::Result<Stream> {
    match entry.transport.as_str() {
        "tcp" => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!(
                "the file names the TCP port {} - this build refuses the TCP link: a peer on an open port cannot be verified against a false server; start the window again with this build",
                entry.address
            ),
        )),
        #[cfg(unix)]
        "unix" => Ok(Stream::Unix(std::os::unix::net::UnixStream::connect(
            &entry.address,
        )?)),
        #[cfg(windows)]
        "pipe" => {
            if entry.address != link::pipe_name(entry.pid) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "the file's pipe name does not belong to process id {}",
                        entry.pid
                    ),
                ));
            }
            let pipe = super::link_pipe::connect(&entry.address)?;
            let server = super::link_pipe::server_pid(&pipe)?;
            if server != entry.pid {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "the pipe is served by process {server}, not the window {} - the link file is stale",
                        entry.pid
                    ),
                ));
            }
            Ok(Stream::Pipe(pipe))
        }
        other => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("transport '{other}' is not served on this platform"),
        )),
    }
}

/// Connect to one window and pass its hello: [`dial`] proves the peer
/// first, then the token tells the window that this client read the
/// discovery file. One elapsed deadline covers the whole exchange
/// (#3874 review): a peer that sends one byte at a time ends on the
/// same clock as a silent one.
///
/// # Errors
/// The connect, the hello or the answer failed inside the deadline.
pub(super) fn open(entry: &Entry) -> std::io::Result<Stream> {
    let deadline = Instant::now() + HELLO_TIMEOUT;
    let stream = dial(entry)?;
    let left = |what: &str| -> std::io::Result<Duration> {
        let rest = deadline.saturating_duration_since(Instant::now());
        if rest.is_zero() {
            Err(timed_out(what))
        } else {
            Ok(rest)
        }
    };
    // The hello write waits for the peer to take it, under the same
    // deadline as everything else in the exchange.
    let mut writer = stream.try_clone()?;
    writer.set_write_timeout(Some(left("the window did not take the hello in time")?))?;
    writeln!(writer, "{}", link::hello_line(&entry.token)).and_then(|()| writer.flush())?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let Some(answer) = read_line_bounded(&mut reader, MAX_ANSWER, Some(deadline))? else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "the peer closed the link before its hello answer",
        ));
    };
    if answer.trim() != link::HELLO_OK {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "the peer did not accept the link token",
        ));
    }
    // A tool may run for minutes (generate): no read deadline from
    // here. The peer proved its transport; its answer lines stay
    // bounded by MAX_ANSWER.
    stream.set_read_timeout(None)?;
    stream.set_write_timeout(None)?;
    Ok(stream)
}

/// One step of a bounded line read.
enum Step {
    /// A complete line, delimiter included.
    Line(String),
    /// The peer hung up.
    Eof,
    /// More bytes needed; the loop reads again.
    More,
}

/// Read one line under a byte cap, and under one elapsed deadline for
/// the whole read when one is given (#3874 review). A line that
/// reaches `cap` bytes without its newline ends the link: no remainder
/// of it is ever read as a new message, so a valid reply hidden in a
/// later piece of an oversized line never counts. `Ok(None)` is a
/// clean close at a line start; a close mid-line is an error.
///
/// With no deadline the caller's own read timeout governs (none, once
/// the peer proved itself: a tool may answer after minutes).
pub(super) fn read_line_bounded(
    reader: &mut BufReader<Stream>,
    cap: u64,
    deadline: Option<Instant>,
) -> std::io::Result<Option<String>> {
    let mut line = Vec::new();
    loop {
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(timed_out(
                "the link peer did not complete its answer in time",
            ));
        }
        if let Some(d) = deadline {
            // One read waits at most the time left to the deadline.
            reader
                .get_ref()
                .set_read_timeout(Some(d.saturating_duration_since(Instant::now())))?;
        }
        let mut used = 0;
        let step = {
            let available = match reader.fill_buf() {
                Ok(b) => b,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock
                            | std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::Interrupted
                    ) =>
                {
                    // Nothing new inside the deadline window: the
                    // check at the loop top ends the wait when it is
                    // due; otherwise the peer may send again.
                    continue;
                }
                Err(e) => return Err(e),
            };
            if available.is_empty() {
                Ok(Step::Eof)
            } else if let Some(at) = available.iter().position(|b| *b == b'\n') {
                line.extend_from_slice(&available[..=at]);
                used = at + 1;
                // A whole line, delimiter included, fits the cap.
                if line.len() > cap as usize {
                    Err(overlong(cap))
                } else {
                    Ok(Step::Line(String::from_utf8_lossy(&line).into_owned()))
                }
            } else {
                line.extend_from_slice(available);
                used = available.len();
                // No delimiter yet: one more byte - the delimiter
                // itself included - cannot keep the line inside the
                // cap, so the link ends here, not after it.
                if line.len() >= cap as usize {
                    Err(overlong(cap))
                } else {
                    Ok(Step::More)
                }
            }
        };
        reader.consume(used);
        match step? {
            Step::Line(text) => return Ok(Some(text)),
            Step::Eof => {
                if line.is_empty() {
                    return Ok(None);
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "the link peer closed the connection mid-line",
                ));
            }
            Step::More => continue,
        }
    }
}

/// Whether the process `pid` runs now. `Some(false)` only when the OS
/// says the pid does not exist; `None` when the check cannot say (an
/// access error) - the caller then treats the window as alive.
#[cfg(unix)]
pub(super) fn pid_alive(pid: u32) -> Option<bool> {
    // A signal of 0 checks existence and sends nothing.
    match unsafe { libc::kill(pid as libc::pid_t, 0) } {
        0 => Some(true),
        -1 => match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ESRCH) => Some(false),
            // EPERM: the process exists, it is just not ours.
            _ => None,
        },
        _ => None,
    }
}

/// The same check through `OpenProcess` on Windows.
#[cfg(windows)]
pub(super) fn pid_alive(pid: u32) -> Option<bool> {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError};
    use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess};
    // PROCESS_QUERY_LIMITED_INFORMATION: enough for the exit code, and
    // allowed on another user's process.
    const QUERY_LIMITED: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;
    const ERROR_INVALID_PARAMETER: u32 = 87;
    unsafe {
        let handle = OpenProcess(QUERY_LIMITED, 0, pid);
        if handle.is_null() {
            return match GetLastError() {
                // No process with this pid exists.
                ERROR_INVALID_PARAMETER => Some(false),
                // Cannot say (access denied): treat the window as alive.
                _ => None,
            };
        }
        let mut code = 0_u32;
        let said = GetExitCodeProcess(handle, &mut code);
        CloseHandle(handle);
        (said != 0).then_some(code == STILL_ACTIVE)
    }
}

/// A platform without a pid check: the caller treats the window as
/// alive, and the transport proof in `open` guards the connection.
#[cfg(not(any(unix, windows)))]
pub(super) fn pid_alive(_pid: u32) -> Option<bool> {
    None
}
