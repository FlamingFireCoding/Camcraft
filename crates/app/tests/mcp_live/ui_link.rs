//! The UI link (#3751, #3874): `camcraft-cli --ui` finds a running
//! window by its discovery file and opens a session with its transport
//! and token. Pinned here: no window is a clear error with exit 3
//! (never a headless run), a wrong token is refused before a line
//! reaches the app, a TCP address is refused by name, a pipe served by
//! another process gets nothing, one deadline ends a slow hello, an
//! oversized line is never split into messages, the file goes when the
//! window's inbox drops, and the bridge's bound reports a truncation.
//! Provenance: our own tests over our own link; no HSMWorks material.

use camcraft_app::mcp::link::{self, Entry};
use camcraft_app::mcp::{Inbox, LinkError, Select, bridge_with_bound, link_call, link_hold};

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn fresh_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("camcraft-ui-link-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn cli_ui(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_camcraft-cli"))
        .env(link::DIR_ENV, dir)
        .arg("--ui")
        .args(args)
        .output()
        .expect("the CLI binary runs")
}

fn write_entry(dir: &Path, entry: &Entry) {
    let text = serde_json::to_string(entry).expect("entry json");
    std::fs::write(dir.join(format!("{}.json", entry.pid)), text).expect("write entry");
}

/// A live pid that is not this test's own: a child that stays up for
/// the whole check (Windows builds it from `ping`, Unix from `sleep`).
fn live_foreign_pid() -> std::process::Child {
    #[cfg(windows)]
    let child = Command::new("ping")
        .args(["-n", "30", "127.0.0.1"])
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("a live pid donor");
    #[cfg(unix)]
    let child = Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("a live pid donor");
    child
}

/// Answer the inbox's requests on this thread (the window's frame loop
/// stand-in) until `client` ends; returns the lines that reached it.
fn serve_until<T: Send + 'static>(
    inbox: &Inbox,
    client: std::thread::JoinHandle<T>,
) -> (T, Vec<String>) {
    serve_until_reply(inbox, client, |msg| {
        json!({
            "jsonrpc": "2.0", "id": msg["id"],
            "result": { "content": [], "structuredContent": { "echo": msg["params"]["name"] }, "isError": false }
        })
        .to_string()
    })
}

/// [`serve_until`] with the window's reply given by `reply`.
fn serve_until_reply<T: Send + 'static>(
    inbox: &Inbox,
    client: std::thread::JoinHandle<T>,
    reply: impl Fn(&Value) -> String,
) -> (T, Vec<String>) {
    let mut lines = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !client.is_finished() {
        assert!(Instant::now() < deadline, "the client did not finish");
        while let Some(req) = inbox.next() {
            let msg: Value = serde_json::from_str(req.line()).expect("json line");
            lines.push(req.line().to_owned());
            req.answer(Some(reply(&msg)));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    (client.join().expect("client thread"), lines)
}

/// Every test of this binary runs in ONE process, and the Windows pipe
/// name comes from the process id: the tests that hold a pipe (a window
/// inbox, or a false window) serialize on this lock, or the second
/// taker of the same name cannot create its instance.
static PIPE_NAME_SLOT: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn pipe_name_slot() -> std::sync::MutexGuard<'static, ()> {
    PIPE_NAME_SLOT.lock().unwrap_or_else(|p| p.into_inner())
}

/// A false window this test process serves on the platform transport
/// (#3874 review). The entry's pid is the test's own, so the
/// transport-level checks pass and the false behaviour is what the CLI
/// sees. What the false window does with the connection is each test's
/// business. The caller holds [`pipe_name_slot`] for the listener's
/// lifetime.
enum FalseWindow {
    #[cfg(windows)]
    Pipe(camcraft_app::mcp::link_pipe::PipeListener),
    #[cfg(unix)]
    Unix(std::os::unix::net::UnixListener),
}

/// One connected CLI, as the false window sees it.
enum FalseClient {
    #[cfg(windows)]
    Pipe(camcraft_app::mcp::link_pipe::PipeStream),
    #[cfg(unix)]
    Unix(std::os::unix::net::UnixStream),
}

impl std::io::Read for FalseClient {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(windows)]
            Self::Pipe(s) => s.read(buf),
            #[cfg(unix)]
            Self::Unix(s) => s.read(buf),
        }
    }
}

impl std::io::Write for FalseClient {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(windows)]
            Self::Pipe(s) => s.write(buf),
            #[cfg(unix)]
            Self::Unix(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(windows)]
            Self::Pipe(s) => s.flush(),
            #[cfg(unix)]
            Self::Unix(s) => s.flush(),
        }
    }
}

/// The transport a false window holds in this test, and the entry that
/// points the CLI at it. The `_dir` is the socket path on Unix; on
/// Windows the pipe name comes from the test's own pid.
fn false_window(_dir: &Path) -> (Entry, FalseWindow) {
    let pid = std::process::id();
    #[cfg(windows)]
    let (transport, address, made) = {
        let name = camcraft_app::mcp::link_pipe::pipe_name(pid);
        let listener =
            camcraft_app::mcp::link_pipe::PipeListener::new(&name).expect("the false pipe");
        ("pipe".to_owned(), name, FalseWindow::Pipe(listener))
    };
    #[cfg(unix)]
    let (transport, address, made) = {
        let path = _dir.join(format!("{pid}.sock"));
        let listener = std::os::unix::net::UnixListener::bind(&path).expect("the false socket");
        listener.set_nonblocking(true).expect("a polling accept");
        let address = path.to_str().expect("temp dir is UTF-8").to_owned();
        ("unix".to_owned(), address, FalseWindow::Unix(listener))
    };
    let entry = Entry {
        format: link::FORMAT,
        pid,
        transport,
        address,
        token: "7".repeat(64),
        project: None,
        app_version: "0".into(),
    };
    (entry, made)
}

/// Take the CLI's connection on a false window, within `wait`.
fn accept_false(window: &FalseWindow, wait: Duration) -> FalseClient {
    let deadline = Instant::now() + wait;
    loop {
        assert!(
            Instant::now() < deadline,
            "no CLI connected to the false window"
        );
        match window {
            #[cfg(windows)]
            FalseWindow::Pipe(l) => {
                if let Ok((s, _)) = l.accept() {
                    return FalseClient::Pipe(s);
                }
            }
            #[cfg(unix)]
            FalseWindow::Unix(l) => {
                if let Ok((s, _)) = l.accept() {
                    return FalseClient::Unix(s);
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn a_cli_call_with_no_window_says_so_and_exits_3() {
    let dir = fresh_dir("none");
    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(3), "{err}");
    assert!(err.contains("no running CamCraft window"), "{err}");
    assert!(err.contains("nothing ran"), "{err}");
    assert!(o.stdout.is_empty(), "no headless answer on stdout");

    // A file a crashed window left behind: its transport does not answer.
    let stale = Entry {
        format: link::FORMAT,
        pid: 4_000_000_000,
        transport: "tcp".into(),
        address: "127.0.0.1:1".into(),
        token: "00".into(),
        project: None,
        app_version: "0".into(),
    };
    write_entry(&dir, &stale);
    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(3), "{err}");
    assert!(err.contains("stale"), "{err}");
    // The dead pid's file is gone: the CLI removed it, so no later run
    // sees the stale entry (never a live window's file, #3874).
    assert!(
        link::entries(&dir).is_empty(),
        "the stale file of a dead pid is removed"
    );

    // A window asked for by a pid that no file names. The stale file is
    // written again, so the listing names a file that is not pid 123.
    write_entry(&dir, &stale);
    let o = cli_ui(&dir, &["--window", "123", "call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(3), "{err}");
    assert!(err.contains("process id 123"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bad_ui_arguments_are_usage_errors() {
    let dir = fresh_dir("usage");
    for args in [
        &["call"][..],
        &["call", "get_job", "[1]"],
        &["call", "get_job", "{nope"],
        &["--window"],
        &["dance"],
    ] {
        let o = cli_ui(&dir, args);
        assert_eq!(
            o.status.code(),
            Some(2),
            "{args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
    let o = cli_ui(&dir, &["--help"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        String::from_utf8_lossy(&o.stdout)
            .contains("Exit: 0 stood, 1 refused, 2 usage, 3 no running window, 4 window busy")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_tcp_link_file_is_refused_by_name() {
    let dir = fresh_dir("tcp");
    // A live pid (this test's own), so the pid filter keeps the file:
    // the refusal must come from the transport itself. A peer on an
    // open TCP port cannot be verified (#3874) - whatever it answers,
    // it first received the token - so this build never dials one.
    let entry = Entry {
        format: link::FORMAT,
        pid: std::process::id(),
        transport: "tcp".into(),
        address: "127.0.0.1:1".into(),
        token: "7".repeat(64),
        project: None,
        app_version: "0".into(),
    };
    write_entry(&dir, &entry);
    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("refuses the TCP link"), "{err}");
    assert!(err.contains("nothing ran"), "{err}");
    assert!(o.stdout.is_empty(), "no answer from a TCP entry");
    assert!(
        link::entries(&dir).len() == 1,
        "a live pid's file is not removed"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[cfg(windows)]
fn a_pipe_served_by_another_process_gets_no_token_and_no_tool_call() {
    use std::io::Read;
    let dir = fresh_dir("false-pipe");
    // The kernel reports the pipe's server process id (#3874): the
    // file names a live foreign pid, but this test's pipe serves the
    // name. The CLI must refuse before it sends anything - not the
    // token, not a tool call - and say which process it found.
    let mut donor = live_foreign_pid();
    let pid = donor.id();
    let name = camcraft_app::mcp::link_pipe::pipe_name(pid);
    let listener = camcraft_app::mcp::link_pipe::PipeListener::new(&name).expect("the false pipe");
    let entry = Entry {
        format: link::FORMAT,
        pid,
        transport: "pipe".into(),
        address: name,
        token: "7".repeat(64),
        project: None,
        app_version: "0".into(),
    };
    write_entry(&dir, &entry);

    let (seen_tx, seen_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let server = std::thread::spawn(move || {
        let mut client = accept_false(&FalseWindow::Pipe(listener), Duration::from_secs(20));
        // The false server would answer anything - even a computed
        // proof - but it never gets the chance: nothing reaches it.
        let mut seen = Vec::new();
        let _ = client.read_to_end(&mut seen);
        seen_tx.send(seen).expect("send what the false server saw");
    });

    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(err.contains("served by process"), "{err}");
    assert!(err.contains(&format!("not the window {pid}")), "{err}");
    assert!(err.contains("nothing ran"), "{err}");
    assert!(o.stdout.is_empty(), "no answer from the false server");

    let seen = seen_rx
        .recv_timeout(Duration::from_secs(20))
        .expect("the false server's capture");
    let text = String::from_utf8_lossy(&seen);
    assert!(
        text.is_empty(),
        "the false server received nothing, not the token: {text}"
    );
    assert!(
        !text.contains("tools/call"),
        "no tool call reached the false server: {text}"
    );
    assert!(
        link::entries(&dir).len() == 1,
        "a live pid's file is not removed"
    );
    let _ = server.join();
    let _ = donor.kill();
    let _ = donor.wait();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_wrong_token_is_refused_and_the_window_goes_on_serving() {
    let _slot = pipe_name_slot();
    let dir = fresh_dir("token");
    let inbox = Inbox::link(&dir, Box::new(|| {})).expect("open the link");
    let entry = inbox.link_entry().expect("a link entry").clone();
    assert_eq!(
        link::entries(&dir),
        vec![entry.clone()],
        "the discovery file names the window"
    );
    assert_eq!(entry.token.len(), 64);

    // The same window, but a file with a wrong token.
    let forged = fresh_dir("token-forged");
    write_entry(
        &forged,
        &Entry {
            token: "f".repeat(64),
            ..entry.clone()
        },
    );
    let forged_dir = forged.clone();
    let client =
        std::thread::spawn(move || link_call(&forged_dir, &Select::Only, "get_job", &json!({})));
    let (result, lines) = serve_until(&inbox, client);
    let err = result.expect_err("a wrong token is refused");
    assert!(matches!(err, LinkError::Failed(_)), "{err:?}");
    assert!(err.to_string().contains("refused the link token"), "{err}");
    assert_eq!(err.exit_code(), 1);
    assert!(
        lines.is_empty(),
        "no line of a refused client reached the app: {lines:?}"
    );

    // A hello that is not JSON at all is refused the same way.
    let wrong_hello = json!({ "camcraft_link": 1 }).to_string();
    assert!(!link::hello_matches(wrong_hello.as_bytes(), &entry.token));
    assert!(!link::hello_matches(b"GET / HTTP/1.1", &entry.token));
    assert!(link::hello_matches(
        link::hello_line(&entry.token).as_bytes(),
        &entry.token
    ));

    // The window still serves a client with the right token.
    let good_dir = dir.clone();
    let client = std::thread::spawn(move || {
        link_call(
            &good_dir,
            &Select::Pid(std::process::id()),
            "get_job",
            &json!({}),
        )
    });
    let (result, lines) = serve_until(&inbox, client);
    let result = result.expect("the right token is served");
    assert_eq!(result["structuredContent"]["echo"], json!("get_job"));
    assert_eq!(
        lines.len(),
        1,
        "one tools/call reached the window: {lines:?}"
    );
    let line: Value = serde_json::from_str(&lines[0]).expect("json");
    assert_eq!(line["method"], json!("tools/call"));

    // The window's file goes with its inbox.
    drop(inbox);
    assert!(
        link::entries(&dir).is_empty(),
        "the discovery file is removed on close"
    );
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&forged);
}

#[test]
fn the_link_file_follows_the_open_project() {
    let _slot = pipe_name_slot();
    let dir = fresh_dir("project");
    let mut inbox = Inbox::link(&dir, Box::new(|| {})).expect("open the link");
    let project = dir.join("part.camproj");
    inbox.note_project(Some(&project));
    let entries = link::entries(&dir);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].project.as_deref(), Some(project.as_path()));
    assert!(Select::Project(project.clone()) != Select::Only);
    assert_eq!(Select::parse("42"), Select::Pid(42));
    assert_eq!(
        Select::parse("a/part.camproj"),
        Select::Project(PathBuf::from("a/part.camproj"))
    );
    drop(inbox);
    let _ = std::fs::remove_dir_all(&dir);
}

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<std::ffi::OsString> {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    move |k: &str| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.into())
}

#[test]
fn the_link_folder_is_the_users_own() {
    let tail = Path::new("camcraft").join("ui-link");
    // An absolute path is spelled per host, so each host checks its own.
    if cfg!(windows) {
        let roaming = r"C:\Users\u\AppData\Roaming";
        assert_eq!(
            link::dir_for("windows", lookup(&[("APPDATA", roaming)])),
            Some(Path::new(roaming).join(&tail))
        );
        assert_eq!(
            link::dir_for("windows", lookup(&[(link::DIR_ENV, r"D:\link")])),
            Some(PathBuf::from(r"D:\link"))
        );
    } else {
        let run = "/run/user/1000";
        let both = [("XDG_RUNTIME_DIR", run), ("HOME", "/home/u")];
        assert_eq!(
            link::dir_for("linux", lookup(&both)),
            Some(Path::new(run).join(&tail)),
            "the runtime folder first"
        );
        assert_eq!(
            link::dir_for("linux", lookup(&[("HOME", "/home/u")])),
            Some(Path::new("/home/u/.local/share").join(&tail))
        );
        assert_eq!(
            link::dir_for("macos", lookup(&[("HOME", "/Users/u")])),
            Some(Path::new("/Users/u/Library/Application Support").join(&tail))
        );
    }
    assert_eq!(
        link::dir_for("linux", lookup(&[])),
        None,
        "no home, no folder"
    );
    assert_eq!(
        link::dir_for("linux", lookup(&[("HOME", "relative")])),
        None,
        "a relative home is refused"
    );
    assert!(link::enabled(None));
    assert!(link::enabled(Some("on")));
    assert!(!link::enabled(Some(" OFF ")));
}

#[test]
fn a_second_ui_call_during_a_slow_call_reports_busy_and_exits_4() {
    let _slot = pipe_name_slot();
    let dir = fresh_dir("busy");
    let inbox = Inbox::link(&dir, Box::new(|| {})).expect("open the link");
    let entry = inbox.link_entry().expect("a link entry").clone();

    // Client 1 holds the window's one-client session: connected, and
    // sending no hello, so the window never becomes free for client 2.
    let held = link_hold(&entry).expect("client 1 holds the session");
    std::thread::sleep(Duration::from_millis(50));

    // Client 2 waits out the whole hello deadline and reports busy.
    let o = cli_ui(&dir, &["call", "get_job"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(4), "{err}");
    assert!(err.contains("is busy with another call"), "{err}");

    drop(held);
    drop(inbox);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_slow_peer_ends_within_the_hello_deadline() {
    use std::io::{Read, Write};
    let _slot = pipe_name_slot();
    let dir = fresh_dir("slow");
    // The whole exchange runs under one elapsed deadline (#3874
    // review): this false window answers the hello one byte every
    // 300 ms with no newline in sight. The CLI must end within the
    // deadline, not one read timeout per byte for ever.
    let (entry, window) = false_window(&dir);
    write_entry(&dir, &entry);
    let started = Instant::now();
    let server = std::thread::spawn(move || {
        let mut client = accept_false(&window, Duration::from_secs(20));
        let mut saw = Vec::new();
        // The hello (or nothing): the trickle is the answer under test.
        let mut chunk = [0_u8; 256];
        let _ = client.read(&mut chunk);
        for _ in 0..60 {
            if client.write_all(b"x").is_err() {
                break; // the CLI hung up
            }
            let _ = client.flush();
            std::thread::sleep(Duration::from_millis(300));
        }
        let _ = client.read_to_end(&mut saw);
        saw
    });
    let o = cli_ui(&dir, &["call", "get_job"]);
    let elapsed = started.elapsed();
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(4), "{err}");
    assert!(err.contains("too slow to answer"), "{err}");
    // The deadline is 5 s; the wait ends with it, not with the trickle.
    assert!(
        elapsed < Duration::from_secs(8),
        "the exchange ran {elapsed:?}"
    );
    let saw = server.join().expect("the false window thread");
    assert!(
        !String::from_utf8_lossy(&saw).contains("tools/call"),
        "no tool call reached the slow peer"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_answer_line_past_the_bound_is_refused_and_never_split() {
    use std::io::{Read, Write};
    let _slot = pipe_name_slot();
    // Two oversized answers (#3874 review): one with a final newline
    // and a valid hello-ok hidden after it, one with neither. The cap
    // (1 MiB, more than any honest answer) must refuse both, and the
    // hidden valid reply must never count - no remainder of an
    // oversized line is read as a new message.
    let over: usize = 2 * 1024 * 1024;
    for tail in [Some(format!("{}\n", link::HELLO_OK)), None] {
        let dir = fresh_dir("oversized");
        let (entry, window) = false_window(&dir);
        write_entry(&dir, &entry);
        let payload = vec![b'x'; over];
        let tail = tail.clone();
        let server = std::thread::spawn(move || {
            let mut client = accept_false(&window, Duration::from_secs(20));
            let mut hello = Vec::new();
            let mut chunk = [0_u8; 256];
            while !hello.contains(&b'\n') {
                if client.read(&mut chunk).unwrap_or(0) == 0 {
                    break;
                }
                hello.extend_from_slice(&chunk);
            }
            // The CLI refuses mid-write once the cap is past: the
            // write breaking is the expected end, not an error.
            let _ = client.write_all(&payload);
            if let Some(tail) = tail.as_deref() {
                let _ = client.write_all(tail.as_bytes());
            }
            let _ = client.flush();
            let mut saw = Vec::new();
            let _ = client.read_to_end(&mut saw);
            saw
        });
        let o = cli_ui(&dir, &["call", "get_job"]);
        let err = String::from_utf8_lossy(&o.stderr);
        assert_eq!(o.status.code(), Some(1), "{err}");
        assert!(err.contains("passed the"), "{err}");
        assert!(err.contains("byte bound without its end"), "{err}");
        assert!(err.contains("nothing ran"), "{err}");
        let saw = server.join().expect("the false window thread");
        assert!(
            !String::from_utf8_lossy(&saw).contains("tools/call"),
            "no tool call reached the false window"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn the_bridge_reports_a_truncated_session_at_its_bound() {
    let _slot = pipe_name_slot();
    // Reaching the bridge's bound with the session still open is a
    // visible failure (#3874 review), never a quiet success after a
    // truncated MCP answer. The bound is a seam, so the test passes a
    // small one and a reply that passes it.
    let dir = fresh_dir("bridge");
    let inbox = Inbox::link(&dir, Box::new(|| {})).expect("open the link");
    let bound: u64 = 16 * 1024;
    let request = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "get_job", "arguments": {} } }).to_string();
    let client_dir = dir.clone();
    let client = std::thread::spawn(move || {
        let input = std::io::Cursor::new(format!("{request}\n").into_bytes());
        let mut output = Vec::new();
        let result = bridge_with_bound(&client_dir, &Select::Only, input, &mut output, bound);
        (result, output)
    });
    let (result, lines) = serve_until_reply(&inbox, client, |msg| {
        json!({
            "jsonrpc": "2.0", "id": msg["id"],
            "result": { "content": [], "structuredContent": { "pad": "y".repeat(96 * 1024) }, "isError": false }
        })
        .to_string()
    });
    let (result, output) = result;
    assert_eq!(
        lines.len(),
        1,
        "one tools/call crossed the bridge: {lines:?}"
    );
    let err = result.expect_err("a truncated session is a failure");
    assert!(matches!(err, LinkError::Failed(_)), "{err:?}");
    assert!(err.to_string().contains("byte bound"), "{err}");
    assert!(err.to_string().contains("truncated"), "{err}");
    assert_eq!(
        err.exit_code(),
        1,
        "the CLI exits 1 on a truncated bridge session"
    );
    assert_eq!(
        output.len(),
        bound as usize,
        "the bound's bytes, no more, were delivered"
    );
    drop(inbox);
    let _ = std::fs::remove_dir_all(&dir);
}
