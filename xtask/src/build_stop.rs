//! What the gate does after a failed test build (#3601): a failed `--no-run` step stops a one-at-a-time plan,
//! the later steps are listed as not run, and a metadata error of rustc gets one diagnostic block.
//! Provenance: our own build tooling (#3601); the rustc error codes E0786 and E0460 from the public rustc error index.

use crate::plan;
use crate::scan::re;

/// The texts that mark a metadata error of rustc: the build output is bad, not the code or the tests.
const SIGNS: [&str; 4] = [
    "found invalid metadata files for crate",
    "E0786",
    "found possibly newer version of crate",
    "E0460",
];

/// True when a failed step named `name` stops a one-at-a-time plan: the whole-gate build and each
/// per-crate test build. The later steps need its binaries, so they can only fail with the same error.
pub fn stops(name: &str) -> bool {
    name == plan::DEBUG_BUILD || name.ends_with(" build (--no-run)")
}

/// The summary note of a step that the failed build `build` stopped.
pub fn not_run_note(build: &str) -> String {
    format!("not run (build failed: {build})")
}

/// The diagnostic block for a failed step whose log `log` has a metadata error; empty for any other log.
/// `args` are the step's cargo arguments (the package after `-p`); `gate` is the gate command line.
pub fn metadata_note(log: &str, args: &[String], gate: &str) -> Result<Vec<String>, String> {
    if !SIGNS.iter().any(|s| log.contains(s)) {
        return Ok(Vec::new());
    }
    let named = re(r"(?:found invalid metadata files for|found possibly newer version of) crate `([^`]+)`")?;
    let krate = named.captures(log).map_or_else(|| "(not named in the log)".to_string(), |c| format!("`{}`", &c[1]));
    let package = args.windows(2).find(|w| w[0] == "-p").map_or("<the package of this step>", |w| w[1].as_str());
    Ok(vec![
        format!("     BUILD FAULT: rustc could not read the metadata of crate {krate} (E0786 / E0460)."),
        "     The build output is bad, not the tests: the test steps of this build did not run.".into(),
        "     Try these, one at a time (#3601):".into(),
        format!("       1. the same gate with fewer jobs: CAMCRAFT_GATE_JOBS=4 cargo xtask {gate}"),
        format!("          (PowerShell: $env:CAMCRAFT_GATE_JOBS=4; cargo xtask {gate})"),
        format!("       2. remove the build of this package, then run the gate again: cargo clean -p {package}"),
    ])
}

/// The gate command line of this process, without the `cargo xtask` part (`gate quick`).
pub fn gate_command() -> String {
    let words: Vec<String> = std::env::args().skip(1).collect();
    if words.is_empty() { "gate quick".into() } else { words.join(" ") }
}
