//! Tests of the end-of-run sweep of superseded test-harness units (#3169).
//! Provenance: our own build tooling (#3169).

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::unit_sweep::{first_live, lines, opted_out, scan, superseded, sweep, Report};

fn tmp(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("xtask-unit-sweep-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}

/// One unit `target/<profile>/build/<pkg>/<hash>/` with the target JSON `json` written at `at`
/// and a 1000-byte output file.
fn unit(root: &Path, profile: &str, pkg: &str, hash: &str, json: &str, at: SystemTime) -> PathBuf {
    let dir = root.join("target").join(profile).join("build").join(pkg).join(hash);
    assert!(fs::create_dir_all(dir.join("fingerprint")).is_ok());
    assert!(fs::create_dir_all(dir.join("out")).is_ok());
    assert!(fs::write(dir.join("out").join("t.exe"), vec![0u8; 1000]).is_ok());
    assert!(fs::write(dir.join("fingerprint").join(format!("dep-{json}")), b"x").is_ok());
    let f = dir.join("fingerprint").join(format!("{json}.json"));
    assert!(fs::write(&f, b"{}").is_ok());
    let file = File::options().write(true).open(&f).expect("open the fingerprint");
    assert!(file.set_modified(at).is_ok());
    dir
}

fn none() -> Option<String> {
    None
}

const T: &str = "test-integration-test-ops";

#[test]
fn an_older_variant_of_a_target_built_in_this_run_is_removed_and_the_newest_survives() {
    let root = tmp("newest");
    let start = SystemTime::now();
    let old = start - Duration::from_secs(3600);
    let new = start + Duration::from_secs(5);
    let a_old = unit(&root, "debug", "camcraft-app", "aaaa", T, old);
    let a_older = unit(&root, "debug", "camcraft-app", "aaab", T, old - Duration::from_secs(60));
    let a_new = unit(&root, "debug", "camcraft-app", "bbbb", T, new);
    let b_old = unit(&root, "debug", "camcraft-app", "cccc", "test-integration-test-cmds", old);
    let b_new = unit(&root, "debug", "camcraft-app", "dddd", "test-integration-test-cmds", new);
    let r = sweep(&root, start, none);
    assert_eq!(r.skipped, None);
    assert_eq!(r.removed, 3, "{r:?}");
    assert!(r.bytes >= 3000, "the report counts the bytes removed: {r:?}");
    assert!(!a_old.exists() && !a_older.exists() && !b_old.exists(), "the superseded variants are removed");
    assert!(a_new.exists() && b_new.exists(), "the newest variant of each target survives");
    assert!(lines(&r)[0].contains("removed 3 superseded test unit dir(s)"), "{:?}", lines(&r));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn two_runs_on_two_trees_leave_one_unit_for_each_target() {
    let root = tmp("bounded");
    let targets = ["test-integration-test-a", "test-integration-test-b", "test-lib-camcraft_app", "test-bin-camcraft-cli"];
    let run1 = SystemTime::now() - Duration::from_secs(7200);
    for (i, t) in targets.iter().enumerate() {
        unit(&root, "debug", "camcraft-app", &format!("1{i}"), t, run1 + Duration::from_secs(1));
    }
    assert!(sweep(&root, run1, none).removed == 0, "the first run has one variant of each target");
    let run2 = SystemTime::now();
    for (i, t) in targets.iter().enumerate() {
        unit(&root, "debug", "camcraft-app", &format!("2{i}"), t, run2 + Duration::from_secs(1));
    }
    assert_eq!(scan(&root.join("target")).len(), 8, "the second tree gave each target a new unit");
    let r = sweep(&root, run2, none);
    assert_eq!(r.removed, 4, "{r:?}");
    assert_eq!(scan(&root.join("target")).len(), targets.len(), "one unit for each target is left");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn no_variant_written_in_this_run_means_nothing_is_removed() {
    let root = tmp("untouched");
    let start = SystemTime::now();
    let a = unit(&root, "debug", "camcraft-motion", "aaaa", T, start - Duration::from_secs(10));
    let b = unit(&root, "debug", "camcraft-motion", "bbbb", T, start - Duration::from_secs(20));
    assert_eq!(sweep(&root, start, none).removed, 0);
    assert!(a.exists() && b.exists(), "a target that this run did not build keeps its variants");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_non_test_unit_is_never_removed() {
    let root = tmp("nontest");
    let start = SystemTime::now();
    let old = start - Duration::from_secs(3600);
    let new = start + Duration::from_secs(5);
    let mut kept = Vec::new();
    for (i, json) in ["lib-camcraft_app", "bin-camcraft-cli", "build-script-build-script-build", "run-build-script-build-script-build"].iter().enumerate() {
        kept.push(unit(&root, "debug", "camcraft-app", &format!("o{i}"), json, old));
        kept.push(unit(&root, "debug", "camcraft-app", &format!("n{i}"), json, new));
    }
    assert!(scan(&root.join("target")).is_empty(), "no lib, bin or build-script unit is a harness unit");
    assert_eq!(sweep(&root, start, none).removed, 0);
    assert!(kept.iter().all(|d| d.exists()));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_held_cargo_lock_or_a_live_process_removes_nothing() {
    let root = tmp("locked");
    let start = SystemTime::now();
    let old = unit(&root, "debug", "camcraft-app", "aaaa", T, start - Duration::from_secs(3600));
    unit(&root, "debug", "camcraft-app", "bbbb", T, start + Duration::from_secs(5));
    let lock = File::create(root.join("target/debug/.cargo-lock")).expect("lock file");
    assert!(lock.try_lock().is_ok(), "the test holds cargo's lock");
    let r = sweep(&root, start, none);
    assert!(r.skipped.as_deref().is_some_and(|w| w.contains("lock")), "{r:?}");
    assert!(old.exists(), "a held lock removes nothing");
    assert!(lines(&r)[0].contains("not run"));
    drop(lock);
    let r = sweep(&root, start, || Some("pid 42".into()));
    assert!(r.skipped.as_deref().is_some_and(|w| w.contains("pid 42")), "{r:?}");
    assert!(old.exists(), "a live process of this tree removes nothing");
    assert_eq!(sweep(&root, start, none).removed, 1, "with no lock and no process the sweep runs");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn superseded_never_lists_the_newest_variant() {
    let root = tmp("pure");
    let start = SystemTime::now();
    unit(&root, "release", "camcraft-sim", "aaaa", T, start + Duration::from_secs(1));
    unit(&root, "release", "camcraft-sim", "bbbb", T, start + Duration::from_secs(2));
    unit(&root, "debug", "camcraft-sim", "cccc", T, start - Duration::from_secs(9));
    let units = scan(&root.join("target"));
    assert_eq!(units.len(), 3);
    assert!(superseded(&units, start).is_empty(), "both release variants are from this run; debug is another profile");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn the_process_check_finds_another_process_of_this_tree_only() {
    let target = if cfg!(windows) { r"C:\W\Tree\target" } else { "/w/tree/target" };
    let other = if cfg!(windows) { r"c:\w\tree\target\debug\xtask.exe gate quick" } else { "/w/tree/target/debug/xtask gate quick" };
    let listing = format!("  1 /usr/bin/cargo xtask gate quick\n 77 {other}\n 99 {other}\n");
    assert_eq!(first_live(&listing, target, 77), Some("pid 99".into()), "own pid is skipped");
    assert_eq!(first_live(&listing, target, 99), Some("pid 77".into()));
    assert_eq!(first_live(" 1 cargo build\n", target, 5), None);
}

#[test]
fn the_sweep_can_be_turned_off_and_reports_failed_deletes_as_warnings() {
    assert!(opted_out(Some("off")) && opted_out(Some("OFF")) && opted_out(Some("0")));
    assert!(!opted_out(None) && !opted_out(Some("on")));
    let r = Report { removed: 1, bytes: 2_500_000_000, failed: vec!["x: denied".into()], skipped: None };
    let l = lines(&r);
    assert!(l[0].contains("2.5 GB"), "{l:?}");
    assert!(l[1].contains("warning: could not remove x: denied"), "{l:?}");
}

#[test]
fn the_gate_summary_runs_the_sweep() {
    let src = include_str!("exec.rs");
    let body = src.split("pub fn summary(").nth(1).unwrap_or_default();
    assert!(body.contains("crate::unit_sweep::after_run(&self.root"), "the summary must run the sweep (#3169)");
}
