//! The CLI's end of the UI link (#3751): find a running window from the
//! discovery files, open a session with its token, and send it MCP
//! `tools/call` lines - the same tool surface `camcraft-cli mcp` serves,
//! answered on the window's UI thread through its History, so Ctrl+Z in
//! the window undoes a CLI step. Never falls back to a headless
//! document: no window is an error by name.
//!
//! Provenance: our own std code over our own link (link.rs); the framing
//! is the MCP stdio transport (modelcontextprotocol.io, revision
//! 2025-06-18). Clean-room, public sources only.

use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};

use super::link::{self, Entry};
use super::wire::Stream;

/// How long a connect, the hello and its proof answer may take before a
/// window is read as not answering (a stale file, a hung window). This
/// deadline covers the whole token-proof exchange: the client sends the
/// hello with its nonce, and the window's answer must carry the proof.
const HELLO_TIMEOUT: Duration = Duration::from_secs(5);

/// The most bytes of one answer line (#3874): a peer that sends one line
/// with no end cannot make the client use all memory. A proof answer or
/// a tool result never approaches the bound. After the proof, no read
/// deadline applies (a tool may run for minutes, #3751) - but the peer
/// proved that it read the discovery file, so a bound on what it may
/// send is what remains to check.
const MAX_ANSWER: u64 = 1024 * 1024;

/// The most bytes a `--ui mcp` bridge session copies from the window to
/// the MCP client, so a peer that never stops writing cannot run for
/// ever. A link session transfers far less.
const MAX_BRIDGE: u64 = 256 * 1024 * 1024;

/// Which window to talk to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Select {
    /// The one running window; an error when there are several.
    Only,
    /// The window of this process id.
    Pid(u32),
    /// The window that has this project open.
    Project(PathBuf),
}

impl Select {
    /// `--window <pid|project path>`: all digits is a process id.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        match text.parse::<u32>() {
            Ok(pid) => Select::Pid(pid),
            Err(_) => Select::Project(PathBuf::from(text)),
        }
    }

    fn admits(&self, entry: &Entry) -> bool {
        match self {
            Select::Only => true,
            Select::Pid(pid) => entry.pid == *pid,
            Select::Project(want) => entry.project.as_deref().is_some_and(|p| same_path(p, want)),
        }
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    a == b || canon(a) == canon(b)
}

/// Why a `--ui` call did not run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkError {
    /// No window answered: nothing ran. Exit 3.
    NoWindow(String),
    /// Several windows answered and none was chosen. Exit 2.
    Choose(String),
    /// The window refused the session or the connection failed. Exit 1.
    Failed(String),
    /// The window is busy with another call. Exit 4.
    Busy(String),
}

impl LinkError {
    /// The CLI's exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            LinkError::NoWindow(_) => 3,
            LinkError::Choose(_) => 2,
            LinkError::Failed(_) => 1,
            LinkError::Busy(_) => 4,
        }
    }
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkError::NoWindow(m)
            | LinkError::Choose(m)
            | LinkError::Failed(m)
            | LinkError::Busy(m) => f.write_str(m),
        }
    }
}

fn describe(entry: &Entry) -> String {
    let project = entry
        .project
        .as_ref()
        .map_or_else(|| "an unsaved document".to_owned(), |p| p.display().to_string());
    format!("pid {} ({project})", entry.pid)
}

/// Connect to one window, pass its token check, and require the proof
/// that the peer mixed the client's fresh nonce with the token (#3874):
/// a peer that answers `HELLO_OK` without the proof never read the
/// discovery file, so it is not the window.
fn open(entry: &Entry) -> std::io::Result<Stream> {
    let stream = match entry.transport.as_str() {
        "tcp" => {
            let addr: std::net::SocketAddr = entry
                .address
                .parse()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "not a socket address"))?;
            // Never send a token off this machine, whatever the file says.
            if !addr.ip().is_loopback() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not a loopback address"));
            }
            Stream::Tcp(std::net::TcpStream::connect_timeout(&addr, HELLO_TIMEOUT)?)
        }
        #[cfg(unix)]
        "unix" => Stream::Unix(std::os::unix::net::UnixStream::connect(&entry.address)?),
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("transport '{other}' is not served on this platform"),
            ));
        }
    };
    stream.set_read_timeout(Some(HELLO_TIMEOUT))?;
    stream.set_write_timeout(Some(HELLO_TIMEOUT))?;
    let mut writer = stream.try_clone()?;
    // The nonce makes the window's proof answer fresh, so a recorded
    // answer of an earlier session proves nothing.
    let nonce = link::new_nonce();
    writeln!(writer, "{}", link::hello_with_nonce(&entry.token, &nonce))?;
    writer.flush()?;
    let mut answer = String::new();
    BufReader::new(stream.try_clone()?).take(MAX_ANSWER).read_line(&mut answer)?;
    if !link::proof_matches(answer.as_bytes(), &entry.token, &nonce) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "the peer did not prove the link token",
        ));
    }
    // A tool may run for minutes (generate): no read deadline from here.
    // The peer proved that it read the discovery file; its answer lines
    // stay bounded by MAX_ANSWER.
    stream.set_read_timeout(None)?;
    stream.set_write_timeout(None)?;
    Ok(stream)
}

/// Whether the process `pid` runs now. `Some(false)` only when the OS
/// says the pid does not exist; `None` when the check cannot say (an
/// access error) - the caller then treats the window as alive.
#[cfg(unix)]
fn pid_alive(pid: u32) -> Option<bool> {
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

#[cfg(windows)]
fn pid_alive(pid: u32) -> Option<bool> {
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

#[cfg(not(any(unix, windows)))]
fn pid_alive(_pid: u32) -> Option<bool> {
    None
}

/// Find the selected window in `dir` and open a session with it.
///
/// # Errors
/// No window answered, several answered without a choice, or the chosen
/// window refused the token.
pub fn connect(dir: &Path, select: &Select) -> Result<(Entry, Stream), LinkError> {
    let all = link::entries(dir);
    if all.is_empty() {
        return Err(LinkError::NoWindow(format!(
            "no running CamCraft window: no link file in {} - start camcraft first (nothing ran)",
            dir.display()
        )));
    }
    let selected: Vec<&Entry> = all.iter().filter(|e| select.admits(e)).collect();
    if selected.is_empty() {
        let running: Vec<String> = all.iter().map(describe).collect();
        let wanted = match select {
            Select::Only => "a window".to_owned(),
            Select::Pid(pid) => format!("process id {pid}"),
            Select::Project(p) => format!("project {}", p.display()),
        };
        return Err(LinkError::NoWindow(format!(
            "no CamCraft window with {wanted}; link files name: {} (nothing ran)",
            running.join(", ")
        )));
    }
    // A file a crashed window left behind (#3874): its pid is dead, so no
    // window serves that socket - and a local process may have bound the
    // port to read the token and the tool calls. The file is removed
    // (never a live window's file) and the entry is skipped. When the
    // check cannot say whether the pid lives, the entry stays and the
    // token proof in `open` guards the connection.
    let candidates: Vec<&Entry> = selected
        .into_iter()
        .filter(|entry| {
            if entry.pid == std::process::id() || pid_alive(entry.pid) != Some(false) {
                return true;
            }
            tracing::info!("ui link: {} is gone; its link file is stale and is removed", describe(entry));
            match std::fs::remove_file(dir.join(format!("{}.json", entry.pid))) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => tracing::warn!("ui link: cannot remove the stale file: {e}"),
            }
            false
        })
        .collect();
    let mut refused = None;
    let mut busy = None;
    let mut open_ones = Vec::new();
    for entry in candidates {
        match open(entry) {
            Ok(stream) => open_ones.push((entry.clone(), stream)),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => refused = Some(describe(entry)),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock) => {
                tracing::info!("ui link: {} is busy: {e}", describe(entry));
                busy = Some(describe(entry));
            }
            // A stale file of a closed window: its socket does not answer.
            Err(e) => tracing::info!("ui link: {} does not answer: {e}", describe(entry)),
        }
    }
    match open_ones.len() {
        1 => open_ones.pop().ok_or_else(|| LinkError::Failed("ui link: lost the open session".into())),
        0 => Err(if let Some(who) = refused {
            LinkError::Failed(format!(
                "the CamCraft window {who} refused the link token in its link file (nothing ran)"
            ))
        } else if let Some(who) = busy {
            LinkError::Busy(format!(
                "the CamCraft window {who} is busy with another call (nothing ran)"
            ))
        } else {
            LinkError::NoWindow(format!(
                "no running CamCraft window answered: the link files in {} are stale (nothing ran)",
                dir.display()
            ))
        }),
        n => {
            let names: Vec<String> = open_ones.iter().map(|(e, _)| describe(e)).collect();
            Err(LinkError::Choose(format!(
                "{n} CamCraft windows are running: {}; choose one with --window <pid|project path> (nothing ran)",
                names.join(", ")
            )))
        }
    }
}

/// Send one `tools/call` to the selected window and return its MCP
/// result (`content`, `structuredContent`, `isError`).
///
/// # Errors
/// See [`connect`]; or the session broke before the answer came.
pub fn call(dir: &Path, select: &Select, tool: &str, args: &Value) -> Result<Value, LinkError> {
    let (entry, stream) = connect(dir, select)?;
    let lost = |e: std::io::Error| LinkError::Failed(format!("the link to {} broke: {e}", describe(&entry)));
    let mut writer = stream.try_clone().map_err(lost)?;
    let line = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": tool, "arguments": args } });
    writeln!(writer, "{line}").and_then(|()| writer.flush()).map_err(lost)?;
    let mut reader = BufReader::new(stream);
    loop {
        let mut text = String::new();
        // Bounded (#3874): one answer line never exceeds MAX_ANSWER.
        let got = reader.by_ref().take(MAX_ANSWER).read_line(&mut text).map_err(lost)?;
        if got == 0 {
            return Err(LinkError::Failed(format!(
                "the window {} closed the link before it answered",
                describe(&entry)
            )));
        }
        let Ok(msg) = serde_json::from_str::<Value>(text.trim()) else { continue };
        // A notification (an idle note) has no id; only our answer counts.
        if msg.get("id") != Some(&json!(1)) {
            continue;
        }
        if let Some(result) = msg.get("result") {
            return Ok(result.clone());
        }
        let why = msg["error"]["message"].as_str().unwrap_or("an unreadable answer");
        return Err(LinkError::Failed(format!("the window refused the call: {why}")));
    }
}

/// `--ui mcp`: a stdio bridge to the selected window's session, for an
/// MCP client that launches stdio servers.
///
/// # Errors
/// See [`connect`]; or a stream failed.
pub fn bridge(dir: &Path, select: &Select, input: impl Read + Send + 'static, mut output: impl Write) -> Result<(), LinkError> {
    let (entry, mut from_app) = connect(dir, select)?;
    let lost = |e: std::io::Error| LinkError::Failed(format!("the link to {} broke: {e}", describe(&entry)));
    let mut to_app = from_app.try_clone().map_err(lost)?;
    // Detached on purpose: a blocked stdin read cannot be interrupted.
    std::thread::Builder::new()
        .name("camcraft-ui-link-bridge".into())
        .spawn(move || {
            let mut input = input;
            if let Err(e) = std::io::copy(&mut input, &mut to_app) {
                tracing::warn!("ui link bridge: sending failed: {e}");
            }
            let _ = to_app.shutdown_write();
        })
        .map_err(lost)?;
    // Bounded (#3874): a peer that never stops writing ends at MAX_BRIDGE
    // bytes, it cannot run for ever.
    let copied = std::io::copy(&mut std::io::Read::by_ref(&mut from_app).take(MAX_BRIDGE), &mut output)
        .map_err(lost)?;
    if copied == MAX_BRIDGE {
        tracing::warn!("ui link bridge: the answer reached the {} byte bound; the copy ended", MAX_BRIDGE);
    }
    Ok(())
}
