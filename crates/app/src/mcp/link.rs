//! The UI link (#3751): each running CamCraft window opens a local
//! socket at start and writes a discovery file, so `camcraft-cli --ui`
//! can find it and send it MCP tool calls that run on the window's own
//! History. This module is the window's end: the folder, the token, the
//! discovery file and the hello line that opens a session.
//!
//! - **Local only.** On Unix the socket is a Unix domain socket in the
//!   user's own folder (mode 0700, the socket 0600). Elsewhere (Windows:
//!   std has no named pipe or local socket) it is TCP bound to 127.0.0.1
//!   with an OS-chosen port; no other interface is ever bound.
//! - **Same user only.** A client must open with the random token from
//!   the discovery file, which only the user can read; any other first
//!   line closes the connection before a byte of it reaches the app.
//! - **One file per window.** `<pid>.json` in the link folder; a window
//!   removes its file when it closes. A file left by a crash is stale:
//!   its socket does not answer, and the client skips it.
//!
//! Provenance: our own design (docs/decisions/3751-cli-ui-link.md) over
//! std; the per-user folders are the platforms' published locations
//! (Windows `%APPDATA%`, macOS `~/Library/Application Support`, the XDG
//! Base Directory spec with `XDG_RUNTIME_DIR` for sockets). Clean-room,
//! public sources only.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::wire::Listener;

/// The override of the link folder: an absolute path (tests use one).
pub const DIR_ENV: &str = "CAMCRAFT_UI_LINK_DIR";
/// `CAMCRAFT_UI_LINK=off` starts a window without its link.
pub const OFF_ENV: &str = "CAMCRAFT_UI_LINK";
/// The discovery file format this build writes and reads.
pub const FORMAT: u32 = 1;
/// The window's answer to a good hello line.
pub const HELLO_OK: &str = r#"{"camcraft_link":1,"ok":true}"#;
/// The window's answer to any other first line; then it hangs up.
pub const HELLO_REFUSED: &str = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32001,"message":"camcraft UI link: wrong or missing token - the connection is closed"}}"#;

/// One window's discovery file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub format: u32,
    /// The window's process id.
    pub pid: u32,
    /// `"unix"` (a socket path) or `"tcp"` (a 127.0.0.1 address).
    pub transport: String,
    pub address: String,
    pub token: String,
    /// The project the window has open, `None` for an unsaved document.
    pub project: Option<PathBuf>,
    pub app_version: String,
}

/// The link folder from an environment lookup, for the named platform
/// (`std::env::consts::OS`). Pure, so the order is testable. `None` = no
/// per-user folder to put one in.
#[must_use]
pub fn dir_for(os: &str, get: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let abs = |k: &str| get(k).map(PathBuf::from).filter(|p| p.is_absolute());
    if let Some(dir) = abs(DIR_ENV) {
        return Some(dir);
    }
    let base = match os {
        "windows" => abs("APPDATA"),
        "macos" => abs("HOME").map(|h| h.join("Library").join("Application Support")),
        // A runtime folder is the user's own (0700) and short, which a
        // socket path needs (about 100 bytes at most).
        _ => abs("XDG_RUNTIME_DIR")
            .or_else(|| abs("XDG_DATA_HOME"))
            .or_else(|| abs("HOME").map(|h| h.join(".local").join("share"))),
    }?;
    Some(base.join("camcraft").join("ui-link"))
}

/// The link folder for this process.
#[must_use]
pub fn dir() -> Option<PathBuf> {
    dir_for(std::env::consts::OS, |k| std::env::var_os(k))
}

/// Whether a window starts its link: on, unless `CAMCRAFT_UI_LINK=off`.
#[must_use]
pub fn enabled(env: Option<&str>) -> bool {
    !env.is_some_and(|v| v.trim().eq_ignore_ascii_case("off"))
}

/// A fresh token: 256 bits of SipHash output under keys that std seeds
/// from the OS's random source (`RandomState`), mixed with the clock and
/// the process id. No new dependency (decision 3751).
fn new_token() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    (0..4_u64)
        .map(|i| {
            let mut h = RandomState::new().build_hasher();
            h.write_u64(i);
            h.write_u128(nanos);
            h.write_u32(std::process::id());
            format!("{:016x}", h.finish())
        })
        .collect()
}

/// A fresh client nonce, from the same randomness as [`new_token`].
#[must_use]
pub fn new_nonce() -> String {
    new_token()
}

/// The first line a client sends.
#[must_use]
pub fn hello_line(token: &str) -> String {
    json!({ "camcraft_link": FORMAT, "token": token }).to_string()
}

/// Whether a first line is the hello with this token. The compare takes
/// the same time for every token of the same length.
#[must_use]
pub fn hello_matches(line: &[u8], token: &str) -> bool {
    let Ok(v) = serde_json::from_slice::<Value>(line) else { return false };
    if v.get("camcraft_link").and_then(Value::as_u64) != Some(u64::from(FORMAT)) {
        return false;
    }
    let Some(got) = v.get("token").and_then(Value::as_str) else { return false };
    got.len() == token.len()
        && got.bytes().zip(token.bytes()).fold(0_u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

/// The first line a client sends, with a nonce (#3874). The window's
/// answer must carry a proof that it mixed this nonce with the token, so
/// a peer that never read the discovery file cannot pass as the window.
#[must_use]
pub fn hello_with_nonce(token: &str, nonce: &str) -> String {
    json!({ "camcraft_link": FORMAT, "token": token, "nonce": nonce }).to_string()
}

/// The nonce a client put in its hello line; `None` when there is none
/// (an older CLI). At most 128 bytes: no honest nonce is longer.
#[must_use]
pub fn hello_nonce(line: &[u8]) -> Option<String> {
    let v = serde_json::from_slice::<Value>(line).ok()?;
    v.get("nonce")?
        .as_str()
        .map(str::to_owned)
        .filter(|n| !n.is_empty() && n.len() <= 128)
}

/// The proof of the token (#3874): two FNV-1a 64-bit words over `token`,
/// a zero byte, `nonce`. This is a knowledge proof, not encryption - the
/// token itself already crosses the same socket in the clear - so a
/// simple published hash is enough. The proof only binds the answer to a
/// peer that read the discovery file and saw this client's fresh nonce.
#[must_use]
pub fn token_proof(token: &str, nonce: &str) -> String {
    fn fnv1a(data: &[u8]) -> u64 {
        // The published FNV-1a 64-bit parameters (public domain).
        data.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01B3)
        })
    }
    let mut data = Vec::with_capacity(token.len() + nonce.len() + 1);
    data.extend_from_slice(token.as_bytes());
    data.push(0);
    data.extend_from_slice(nonce.as_bytes());
    let a = fnv1a(&data);
    let b = fnv1a(&a.to_be_bytes());
    format!("{a:016x}{b:016x}")
}

/// The window's answer to a good hello: plain [`HELLO_OK`] for a hello
/// without a nonce (an older CLI), otherwise with the proof of the token
/// and the client's nonce.
#[must_use]
pub fn hello_ok_line(token: &str, nonce: Option<&str>) -> String {
    match nonce {
        None => HELLO_OK.to_owned(),
        Some(n) => json!({ "camcraft_link": FORMAT, "ok": true, "proof": token_proof(token, n) })
            .to_string(),
    }
}

/// Whether an answer proves that the peer knows `token` and mixed in the
/// client's `nonce`. The compare takes the same time for equal lengths.
#[must_use]
pub fn proof_matches(answer: &[u8], token: &str, nonce: &str) -> bool {
    let Ok(v) = serde_json::from_slice::<Value>(answer) else { return false };
    if v.get("camcraft_link").and_then(Value::as_u64) != Some(u64::from(FORMAT)) {
        return false;
    }
    if v.get("ok").and_then(Value::as_bool) != Some(true) {
        return false;
    }
    let want = token_proof(token, nonce);
    let Some(got) = v.get("proof").and_then(Value::as_str) else { return false };
    got.len() == want.len()
        && got.bytes().zip(want.bytes()).fold(0_u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

/// A window's registration: its discovery file (and socket file), removed
/// when it drops.
#[derive(Debug)]
pub struct Registration {
    file: PathBuf,
    socket: Option<PathBuf>,
    entry: Entry,
}

impl Registration {
    #[must_use]
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    /// Record the window's open project, so `--window <project>` finds it.
    /// Writes only when it changed; a failed write is logged.
    pub fn note_project(&mut self, project: Option<&Path>) {
        if self.entry.project.as_deref() == project {
            return;
        }
        self.entry.project = project.map(Path::to_path_buf);
        if let Err(e) = write_entry(&self.file, &self.entry) {
            tracing::warn!("ui link: cannot update {}: {e}", self.file.display());
        }
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        for path in std::iter::once(&self.file).chain(self.socket.as_ref()) {
            if let Err(e) = std::fs::remove_file(path)
                && e.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!("ui link: cannot remove {}: {e}", path.display());
            }
        }
    }
}

/// Bind this window's socket in `dir` and write its discovery file.
///
/// # Errors
/// The folder cannot be made private, the bind failed, or the file
/// could not be written (then nothing is left behind).
pub(crate) fn open(dir: &Path) -> std::io::Result<(Listener, Registration, String)> {
    prepare_dir(dir)?;
    let pid = std::process::id();
    let token = new_token();
    let (listener, transport, address, socket) = bind(dir, pid)?;
    let entry = Entry {
        format: FORMAT,
        pid,
        transport: transport.to_owned(),
        address,
        token: token.clone(),
        project: None,
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
    };
    // Built before the write, so a failed write still removes the socket.
    let registration = Registration { file: dir.join(format!("{pid}.json")), socket, entry };
    write_entry(&registration.file, &registration.entry)?;
    Ok((listener, registration, token))
}

fn prepare_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(unix)]
fn bind(dir: &Path, pid: u32) -> std::io::Result<(Listener, &'static str, String, Option<PathBuf>)> {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(format!("{pid}.sock"));
    let address = path
        .to_str()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "the link folder path is not UTF-8"))?
        .to_owned();
    // A file of a crashed process with the same pid: nobody serves it.
    let _ = std::fs::remove_file(&path);
    let listener = std::os::unix::net::UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok((Listener::Unix(listener), "unix", address, Some(path)))
}

#[cfg(not(unix))]
fn bind(_dir: &Path, _pid: u32) -> std::io::Result<(Listener, &'static str, String, Option<PathBuf>)> {
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    let address = listener.local_addr()?.to_string();
    Ok((Listener::Tcp(listener), "tcp", address, None))
}

/// Write the file all or nothing (a temp file, then a rename), readable
/// by the user only on Unix.
fn write_entry(file: &Path, entry: &Entry) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(entry).map_err(std::io::Error::other)?;
    let tmp = file.with_extension("json.tmp");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut out = options.open(&tmp)?;
    out.write_all(text.as_bytes())?;
    out.sync_all()?;
    drop(out);
    std::fs::rename(&tmp, file)
}

/// Every readable discovery file in `dir`, by process id. A file that is
/// not one (a temp file, another format) is skipped and logged.
#[must_use]
pub fn entries(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<Entry> = read
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| {
            let parsed = std::fs::read_to_string(&p)
                .ok()
                .and_then(|t| serde_json::from_str::<Entry>(&t).ok())
                .filter(|e| e.format == FORMAT);
            if parsed.is_none() {
                tracing::info!("ui link: {} is not a link file of this version; skipped", p.display());
            }
            parsed
        })
        .collect();
    found.sort_by_key(|e| e.pid);
    found
}
