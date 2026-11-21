//! The end-of-run sweep of superseded test-harness units in the gate's own `target/` (#3169).
//! Provenance: our own build tooling (#3150, #3169); the Cargo book, "Build cache" (one directory for
//! each unit, `build/<package>/<hash>/`, named by a hash of its profile, features and dependencies).
//!
//! Why: each change of the unit env or of a feature resolution gives every test executable of a crate a
//! new unit directory, and cargo never removes the old one. `camcraft-app` alone has about 620
//! integration test binaries of about 100 MB each, so each such change adds about 60 GB. The sweep
//! removes only a test-harness unit (its fingerprint names a `test-lib`, `test-integration-test` or
//! `test-bin` target) when this run wrote a newer variant of the same target. Libraries and binaries
//! stay: two feature resolutions of one library can both be valid in one tree.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

/// `CAMCRAFT_GATE_SWEEP=off` (or `0`) turns the sweep off.
pub const OPT_OUT: &str = "CAMCRAFT_GATE_SWEEP";

/// The fingerprint file prefixes of the test-harness targets; nothing else is ever removed.
const HARNESS: [&str; 3] = ["test-lib-", "test-integration-test-", "test-bin-"];

/// The cargo profile directories that the gate builds into.
const PROFILES: [&str; 2] = ["debug", "release"];

/// One test-harness unit directory: its target key (profile, package, fingerprint file) and the
/// time cargo last wrote its fingerprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub dir: PathBuf,
    pub key: (String, String, String),
    pub written: SystemTime,
}

/// What the sweep did: removed directories and bytes, failed deletes, or why it did not run.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub removed: usize,
    pub bytes: u64,
    pub failed: Vec<String>,
    pub skipped: Option<String>,
}

/// The name of the one target JSON in `fingerprint/` when it is a test-harness target, else `None`.
/// A directory with any other target JSON (a lib, a bin, a build script) is not a harness unit.
fn harness_json(fingerprint: &Path) -> Option<(String, SystemTime)> {
    let mut found = None;
    for e in fs::read_dir(fingerprint).ok()?.filter_map(Result::ok) {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        if found.is_some() || !HARNESS.iter().any(|h| name.starts_with(h)) {
            return None;
        }
        let written = e.metadata().and_then(|m| m.modified()).ok()?;
        found = Some((name, written));
    }
    found
}

/// The test-harness units under `target/<profile>/build/<package>/<hash>/` of `target`.
pub fn scan(target: &Path) -> Vec<Unit> {
    let mut out = Vec::new();
    for profile in PROFILES {
        let build = target.join(profile).join("build");
        for package in fs::read_dir(&build).into_iter().flatten().filter_map(Result::ok) {
            let pkg = package.file_name().to_string_lossy().into_owned();
            for unit in fs::read_dir(package.path()).into_iter().flatten().filter_map(Result::ok) {
                let dir = unit.path();
                if let Some((json, written)) = harness_json(&dir.join("fingerprint")) {
                    out.push(Unit { dir, key: (profile.into(), pkg.clone(), json), written });
                }
            }
        }
    }
    out
}

/// The units to remove: of each target that has a variant written at or after `start`, every variant
/// written before `start`. The newest variant of a target is never in the list.
pub fn superseded(units: &[Unit], start: SystemTime) -> Vec<&Unit> {
    let mut groups: BTreeMap<&(String, String, String), Vec<&Unit>> = BTreeMap::new();
    for u in units {
        groups.entry(&u.key).or_default().push(u);
    }
    let mut out = Vec::new();
    for (_, g) in groups {
        if g.len() < 2 || !g.iter().any(|u| u.written >= start) {
            continue;
        }
        out.extend(g.into_iter().filter(|u| u.written < start));
    }
    out
}

fn size(path: &Path) -> u64 {
    let Ok(m) = fs::symlink_metadata(path) else { return 0 };
    if !m.is_dir() {
        return m.len();
    }
    fs::read_dir(path).into_iter().flatten().filter_map(Result::ok).map(|e| size(&e.path())).sum()
}

/// True when the variable turns the sweep off.
pub fn opted_out(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.eq_ignore_ascii_case("off") || v == "0")
}

/// The reason not to sweep when the gate's target directory is not `root/target`: then the gate cannot
/// know which directory it built into.
fn foreign_target() -> Option<String> {
    ["CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "CARGO_BUILD_BUILD_DIR"]
        .into_iter()
        .find(|k| std::env::var_os(k).is_some_and(|v| !v.is_empty()))
        .map(|k| format!("{k} is set, so the gate's target directory is not known"))
}

/// Removes the superseded units of `root/target` (see `superseded`). `live` names a live cargo, rustc,
/// xtask or test process of this tree (`None`: none). Cargo's locks of each profile directory are held
/// while it deletes; a held lock or a live process skips the sweep. A failed delete is a warning.
pub fn sweep(root: &Path, start: SystemTime, live: impl Fn() -> Option<String>) -> Report {
    let target = root.join("target");
    let mut locks = Vec::new();
    for p in PROFILES {
        let dir = target.join(p);
        if !dir.is_dir() {
            continue;
        }
        match crate::seed::lock(&dir, false) {
            Ok(Some(held)) => locks.push(held),
            Ok(None) => return skipped(format!("a cargo lock in target/{p} is held")),
            Err(e) => return skipped(e),
        }
    }
    if let Some(who) = live() {
        return skipped(format!("a live process uses this tree ({who})"));
    }
    let units = scan(&target);
    let mut report = Report::default();
    for u in superseded(&units, start) {
        let bytes = size(&u.dir);
        match fs::remove_dir_all(&u.dir) {
            Ok(()) => {
                report.removed += 1;
                report.bytes += bytes;
            }
            Err(e) => report.failed.push(format!("{}: {e}", u.dir.display())),
        }
    }
    drop(locks);
    report
}

fn skipped(why: String) -> Report {
    Report { skipped: Some(why), ..Report::default() }
}

/// The sweep of a finished gate run that started `elapsed` ago: off by `OPT_OUT` or a foreign target
/// directory, else `sweep` with the process check of this machine. A start time that cannot be computed
/// counts as the epoch: every unit is then "written in this run", and nothing is removed.
pub fn after_run(root: &Path, elapsed: Duration) -> Report {
    let start = SystemTime::now().checked_sub(elapsed).unwrap_or(SystemTime::UNIX_EPOCH);
    if opted_out(std::env::var(OPT_OUT).ok().as_deref()) {
        return skipped(format!("{OPT_OUT} is off"));
    }
    if let Some(why) = foreign_target() {
        return skipped(why);
    }
    sweep(root, start, || live_process(root))
}

/// The summary lines of a sweep: what it removed (count and GB), each failed delete as a warning.
pub fn lines(r: &Report) -> Vec<String> {
    if let Some(why) = &r.skipped {
        return vec![format!("  unit sweep: not run ({why})")];
    }
    let gb = r.bytes as f64 / 1e9;
    let mut out = vec![format!(
        "  unit sweep: removed {} superseded test unit dir(s), {gb:.1} GB (#3169; {OPT_OUT}=off turns it off)",
        r.removed
    )];
    out.extend(r.failed.iter().map(|f| format!("  unit sweep: warning: could not remove {f}")));
    out
}

/// A path in the form that the process check compares: `/` separators, no verbatim prefix, and on
/// Windows lower case.
pub fn norm(s: &str) -> String {
    let s = s.replace('\\', "/");
    let s = s.strip_prefix("//?/").unwrap_or(&s).to_string();
    if cfg!(windows) { s.to_lowercase() } else { s }
}

/// The first line of `listing` (`<pid> <command line>`, one process per line) whose command line names
/// `target` and whose pid is not `me`.
pub fn first_live(listing: &str, target: &str, me: u32) -> Option<String> {
    let needle = norm(target);
    listing.lines().map(str::trim).find_map(|l| {
        let (pid, cmd) = l.split_once(char::is_whitespace)?;
        let pid: u32 = pid.trim().parse().ok()?;
        (pid != me && norm(cmd).contains(&needle)).then(|| format!("pid {pid}"))
    })
}

/// A live process other than this one whose command line names `root/target` (another gate's xtask,
/// a cargo or rustc writing there, a test executable). When the process list cannot be read, the
/// answer is "live": the sweep then does nothing.
fn live_process(root: &Path) -> Option<String> {
    let out = if cfg!(windows) {
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command"])
            .arg("Get-CimInstance Win32_Process | ForEach-Object { \"$($_.ProcessId) $($_.CommandLine)\" }")
            .output()
    } else {
        Command::new("ps").args(["-eo", "pid=,args="]).output()
    };
    match out {
        Ok(o) if o.status.success() => {
            let target = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf()).join("target");
            first_live(&String::from_utf8_lossy(&o.stdout), &target.to_string_lossy(), std::process::id())
        }
        _ => Some("the process list could not be read".into()),
    }
}
