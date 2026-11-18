//! Known-fail heavy tests pinned to an open, blocked issue: the release heavy lanes skip them by name, and one
//! extra step for each runs it alone and must see it fail, so a skip that starts to pass fails the gate.
//! Provenance: our own build law (AGENTS.md "Three gate tiers"; the #1950 known-fail decision, RECOMMENDED.md);
//! libtest's public `--exact` / `--skip` / `--ignored` flags and the `gh issue view` CLI.

use std::path::Path;
use std::process::Command;

/// One `#[ignore]`d test that is red on purpose: it pins the target of an open issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownFail {
    /// Crate dir (`motion`, `app`).
    pub dir: &'static str,
    /// Heavy target label: `lib` or the integration binary's name.
    pub target: &'static str,
    /// The libtest path of the test inside that target.
    pub test: &'static str,
    /// The open issue the test pins; the gate refuses the skip once the issue reads closed.
    pub issue: u32,
}

/// Only these tests. Each entry needs a `docs/decisions/` entry and an open issue; delete the entry when the
/// test passes (its known-fail step then reads FAIL and says so).
pub const KNOWN_FAIL: &[KnownFail] = &[
    KnownFail {
        dir: "motion",
        target: "lib",
        test: "machining2d::pocket_hsm::ring_tests::the_stock_ring_level_reaches_the_air_planner_with_the_stock",
        issue: 1950,
    },
    KnownFail {
        dir: "motion",
        target: "lib",
        test: "machining2d::pocket_hsm::ring_tests::the_stock_ring_level_plans_adaptive_with_no_budget_note",
        issue: 1950,
    },
    KnownFail {
        dir: "app",
        target: "pocket3d_hsm",
        test: "the_users_whole_model_bore_roughs_adaptive_on_each_level",
        issue: 1950,
    },
];

/// The known-fail tests of one heavy target.
pub fn of(dir: &str, target: &str) -> Vec<&'static KnownFail> {
    KNOWN_FAIL.iter().filter(|k| k.dir == dir && k.target == target).collect()
}

/// The libtest args (before `--test-threads`/`--ignored`) that leave the known-fail tests of a target out.
pub fn skip_args(dir: &str, target: &str) -> Vec<String> {
    let mut out = Vec::new();
    for k in of(dir, target) {
        if out.is_empty() {
            out.push("--exact".to_string());
        }
        out.push("--skip".to_string());
        out.push(k.test.to_string());
    }
    out
}

/// The line the release plan prints for each skipped test.
pub fn note(k: &KnownFail) -> String {
    format!(
        "SKIPPED (known fail, #{}): {} heavy: {} {} - its own step must still fail",
        k.issue, k.dir, k.target, k.test
    )
}

/// The short step name of a known-fail check.
pub fn step_name(k: &KnownFail) -> String {
    let short = k.test.rsplit("::").next().unwrap_or(k.test);
    format!("{} heavy: known fail #{} {}", k.dir, k.issue, short)
}

/// Reads each pinned issue's state with `gh`. A closed issue is an error (the skip is stale); an issue that
/// `gh` cannot read gives a note and no verdict, so the known-fail steps stay the guard.
pub fn check_issues(root: &Path, dry: bool) -> Result<Vec<String>, String> {
    let mut issues: Vec<u32> = KNOWN_FAIL.iter().map(|k| k.issue).collect();
    issues.sort_unstable();
    issues.dedup();
    let mut notes = Vec::new();
    for n in issues {
        if dry {
            notes.push(format!("known fail: #{n} state not read (dry run)"));
            continue;
        }
        let out = Command::new("gh")
            .args(["issue", "view", &n.to_string(), "--json", "state", "-q", ".state"])
            .current_dir(root)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if state.eq_ignore_ascii_case("closed") {
                    return Err(format!(
                        "known fail: #{n} is closed, but xtask/src/known_fail.rs still skips its tests - remove the entries"
                    ));
                }
                notes.push(format!("known fail: #{n} is {state}"));
            }
            _ => notes.push(format!(
                "known fail: could not read #{n} with gh - issue state not checked; the known-fail steps still must fail"
            )),
        }
    }
    Ok(notes)
}
