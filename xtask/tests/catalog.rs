//! Pins `cargo xtask catalog` CLI contract, checks, and safety refusals (#2393).
//! Provenance: CamCraft example catalog build/regression tooling tests.

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn xtask_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xtask")
}

#[test]
fn help_lists_catalog_command() {
    let out = Command::new(xtask_bin())
        .arg("--help")
        .output()
        .expect("run xtask --help");
    assert!(out.status.success(), "--help should succeed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("catalog"),
        "USAGE must list `cargo xtask catalog`, got:\n{stdout}"
    );
}

#[test]
fn catalog_check_verifies_seed_case() {
    let out = Command::new(xtask_bin())
        .current_dir(workspace_root())
        .args(["catalog", "--case", "2d/first-part", "--check"])
        .output()
        .expect("run xtask catalog --check");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "catalog --check failed (status {:?})\nstdout:\n{stdout}\nstderr:\n{stderr}",
        out.status.code()
    );
    assert!(
        stdout.contains("checked 2d/first-part"),
        "expected checked 2d/first-part output, got:\n{stdout}"
    );
}

#[test]
fn catalog_refuses_unsafe_branch_and_in_tree_output_and_bad_flags() {
    let root = workspace_root();
    for (args, needle) in [
        (&["catalog", "--screenshots", "--screenshots-branch", "main"][..], "main"),
        (&["catalog", "--screenshots", "--screenshots-out", "docs/inside"][..], "outside"),
        (&["catalog", "--with-guide"][..], "--screenshots"),
        (&["catalog", "--check", "--rebuild"][..], "--check"),
        (&["catalog", "--bogus-flag"][..], "--bogus-flag"),
    ] {
        let out = Command::new(xtask_bin())
            .current_dir(&root)
            .args(args)
            .output()
            .expect("run xtask catalog");
        assert!(!out.status.success(), "expected failure for {args:?}");
        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            combined.contains(needle),
            "expected refusal naming '{needle}' for {args:?}, got:\n{combined}"
        );
    }
}
