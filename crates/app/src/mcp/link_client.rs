//! The CLI's end of the UI link (#3751): find a running window from the
//! discovery files, open a session with its transport and token, and
//! send it MCP `tools/call` lines - the same tool surface
//! `camcraft-cli mcp` serves, answered on the window's UI thread
//! through its History, so Ctrl+Z in the window undoes a CLI step.
//! Never falls back to a headless document: no window is an error by
//! name.
//!
//! Who the peer is (#3874) is `link_peer`'s law: the transport proves
//! the peer before the token crosses, the hello runs under one elapsed
//! deadline, and every answer line is read under a byte cap. This
//! module is the CLI's own surface on top of it: the window choice, the
//! one-call verb, and the stdio bridge.
//!
//! Provenance: our own std code over our own link (link.rs, link_peer,
//! link_pipe.rs); the framing is the MCP stdio transport
//! (modelcontextprotocol.io, revision 2025-06-18). Clean-room, public
//! sources only.

use std::fmt;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::link::{self, Entry};
use super::link_peer::{self, read_line_bounded};
use super::wire::Stream;

/// The most bytes a `--ui mcp` bridge session copies from the window to
/// the MCP client, so a peer that never stops writing cannot run for
/// ever. Reaching the bound with the session still open is a visible
/// failure, never a quiet success after a truncated answer.
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
    /// The window is busy with another call, or too slow to answer.
    /// Exit 4.
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
    let project = entry.project.as_ref().map_or_else(
        || "an unsaved document".to_owned(),
        |p| p.display().to_string(),
    );
    format!("pid {} ({project})", entry.pid)
}

/// A transport connection held open with no hello - the busy test uses
/// it to occupy the window's one-client session. Drop it to let go.
pub struct HeldSession {
    stream: Stream,
}

impl Drop for HeldSession {
    fn drop(&mut self) {
        // Let a socket window see the session end at once; a pipe
        // window notices through the broken pipe instead.
        let _ = self.stream.shutdown_write();
    }
}

/// [`HeldSession`]: the transport connection of `entry`, open and idle.
///
/// # Errors
/// See [`link_peer::dial`].
pub fn hold(entry: &Entry) -> std::io::Result<HeldSession> {
    link_peer::dial(entry).map(|stream| HeldSession { stream })
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
    // A file a crashed window left behind (#3874): its pid is dead, so
    // no window serves that transport. The file is removed (never a
    // live window's file) and the entry is skipped. When the check
    // cannot say whether the pid lives, the entry stays; the
    // transport proof in `link_peer::open` guards the connection.
    let candidates: Vec<&Entry> = selected
        .into_iter()
        .filter(|entry| {
            if entry.pid == std::process::id() || link_peer::pid_alive(entry.pid) != Some(false) {
                return true;
            }
            tracing::info!(
                "ui link: {} is gone; its link file is stale and is removed",
                describe(entry)
            );
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
    let mut unserved = None;
    let mut open_ones = Vec::new();
    for entry in candidates {
        match link_peer::open(entry) {
            Ok(stream) => open_ones.push((entry.clone(), stream)),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                refused = Some(describe(entry))
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                tracing::info!("ui link: {} is busy or too slow: {e}", describe(entry));
                busy = Some(describe(entry));
            }
            // A file this build refuses to dial: a TCP port, or a pipe
            // that another process serves. The reason is the report.
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::Unsupported | std::io::ErrorKind::InvalidData
                ) =>
            {
                unserved = Some(format!("{}, refused: {e}", describe(entry)));
            }
            // A stale file of a closed window: its transport does not answer.
            Err(e) => tracing::info!("ui link: {} does not answer: {e}", describe(entry)),
        }
    }
    match open_ones.len() {
        1 => open_ones
            .pop()
            .ok_or_else(|| LinkError::Failed("ui link: lost the open session".into())),
        0 => Err(if let Some(who) = refused {
            LinkError::Failed(format!(
                "the CamCraft window {who} refused the link token in its link file (nothing ran)"
            ))
        } else if let Some(who) = busy {
            LinkError::Busy(format!(
                "the CamCraft window {who} is busy with another call or too slow to answer (nothing ran)"
            ))
        } else if let Some(what) = unserved {
            LinkError::Failed(format!("the link file of {what} (nothing ran)"))
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
    let lost = |e: std::io::Error| {
        LinkError::Failed(format!("the link to {} broke: {e}", describe(&entry)))
    };
    let mut writer = stream.try_clone().map_err(lost)?;
    let line = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": tool, "arguments": args } });
    writeln!(writer, "{line}")
        .and_then(|()| writer.flush())
        .map_err(lost)?;
    let mut reader = BufReader::new(stream);
    loop {
        // Bounded (#3874): one answer line never exceeds MAX_ANSWER, a
        // line that reaches the bound ends the link, and no remainder
        // of an oversized line is read as a new message.
        let Some(text) =
            read_line_bounded(&mut reader, link_peer::MAX_ANSWER, None).map_err(lost)?
        else {
            return Err(LinkError::Failed(format!(
                "the window {} closed the link before it answered",
                describe(&entry)
            )));
        };
        let Ok(msg) = serde_json::from_str::<Value>(text.trim()) else {
            continue;
        };
        // A notification (an idle note) has no id; only our answer counts.
        if msg.get("id") != Some(&json!(1)) {
            continue;
        }
        if let Some(result) = msg.get("result") {
            return Ok(result.clone());
        }
        let why = msg["error"]["message"]
            .as_str()
            .unwrap_or("an unreadable answer");
        return Err(LinkError::Failed(format!(
            "the window refused the call: {why}"
        )));
    }
}

/// `--ui mcp`: a stdio bridge to the selected window's session, for an
/// MCP client that launches stdio servers.
///
/// # Errors
/// See [`connect`]; or a stream failed.
pub fn bridge(
    dir: &Path,
    select: &Select,
    input: impl Read + Send + 'static,
    mut output: impl Write,
) -> Result<(), LinkError> {
    bridge_with_bound(dir, select, input, &mut output, MAX_BRIDGE)
}

/// [`bridge`] with the copy bound given: the bound is a test seam, so a
/// test can exceed it without writing [`MAX_BRIDGE`] bytes.
///
/// # Errors
/// See [`bridge`]; or the bound ended a session that was still open.
pub fn bridge_with_bound(
    dir: &Path,
    select: &Select,
    input: impl Read + Send + 'static,
    output: &mut impl Write,
    max: u64,
) -> Result<(), LinkError> {
    let (entry, mut from_app) = connect(dir, select)?;
    let lost = |e: std::io::Error| {
        LinkError::Failed(format!("the link to {} broke: {e}", describe(&entry)))
    };
    let mut to_app = from_app.try_clone().map_err(lost)?;
    // Detached on purpose: a blocked stdin read cannot be interrupted.
    // A pipe has no half-close, so this shutdown signals only a socket
    // transport; a pipe session ends through the idle drop instead.
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
    // Bounded (#3874): a peer that never stops writing ends at the
    // bound - and the bound is a visible failure when the session was
    // still open, never a quiet success after a truncated answer.
    let copied =
        std::io::copy(&mut std::io::Read::by_ref(&mut from_app).take(max), output).map_err(lost)?;
    if copied == max {
        // The take hid whether the peer had more to send: one more
        // byte says. A clean close there is a complete transfer.
        let mut probe = [0_u8; 1];
        if from_app.read(&mut probe).map_err(lost)? > 0 {
            return Err(LinkError::Failed(format!(
                "the link to {} passed the {max} byte bound with the session still open - the answer is truncated and the session ended (nothing ran)",
                describe(&entry)
            )));
        }
    }
    Ok(())
}
