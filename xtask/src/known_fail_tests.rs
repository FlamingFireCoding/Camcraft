//! Pins the known-fail skip of the release heavy lanes (the #1950 known-fail decision, RECOMMENDED.md).
//! Provenance: our own build law (AGENTS.md "Three gate tiers"); libtest's `--exact` / `--skip` flags.

use std::path::Path;

use crate::known_fail::{self, KNOWN_FAIL};
use crate::{exec, plan, scan};

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn each_known_fail_is_skipped_in_its_lane_and_checked_alone() {
    let steps = plan::heavy_steps(&root());
    for k in KNOWN_FAIL {
        let lane = steps.iter().find(|s| s.name == format!("{} heavy: {}", k.dir, k.target)).expect("lane");
        assert!(lane.args.windows(2).any(|w| w == ["--skip", k.test]), "{}: {:?}", k.test, lane.args);
        assert!(lane.args.contains(&"--exact".to_string()) && !lane.expect_fail);
        let check: Vec<_> = steps.iter().filter(|s| s.name == known_fail::step_name(k)).collect();
        assert_eq!(check.len(), 1, "{}", k.test);
        assert!(check[0].expect_fail);
        assert!(check[0].args.ends_with(&["--".into(), plan::T.into(), "--exact".into(), k.test.into(), "--ignored".into()]));
        assert!(!check[0].args.iter().any(|a| a == "--skip"));
    }
    // Only the known-fail checks expect a failure, and only the lanes that hold one skip anything.
    assert_eq!(steps.iter().filter(|s| s.expect_fail).count(), KNOWN_FAIL.len());
    let skipping = steps.iter().filter(|s| s.args.iter().any(|a| a == "--skip")).count();
    let lanes: std::collections::BTreeSet<_> = KNOWN_FAIL.iter().map(|k| (k.dir, k.target)).collect();
    assert_eq!(skipping, lanes.len());
}

#[test]
fn each_known_fail_is_an_ignored_test_that_names_its_issue() {
    for k in KNOWN_FAIL {
        let name = k.test.rsplit("::").next().unwrap_or(k.test);
        let dir = root().join("crates").join(k.dir);
        let files = if k.target == "lib" {
            scan::rs_files_recursive(&dir.join("src"))
        } else {
            vec![dir.join("tests").join(format!("{}.rs", k.target))]
        };
        let hit = files.iter().map(|f| scan::read(f)).find_map(|text| {
            let (before, _) = text.split_once(&format!("fn {name}()"))?;
            Some(before.rsplit("#[test]").next().unwrap_or_default().to_string())
        });
        let attrs = hit.unwrap_or_else(|| panic!("{} not found in {}/{}", k.test, k.dir, k.target));
        assert!(attrs.contains("#[ignore = \""), "{} must stay #[ignore]d", k.test);
        assert!(attrs.contains(&format!("#{}", k.issue)), "{}: ignore reason must name #{}", k.test, k.issue);
    }
}

#[test]
fn a_known_fail_step_is_ok_only_when_its_test_fails() {
    let mut red = "failed: unittests src/lib.rs [tests: x]".to_string();
    assert!(exec::expect_fail_note(101, &mut red));
    assert!(red.starts_with("known fail, red as expected"));
    let mut green = String::new();
    assert!(!exec::expect_fail_note(0, &mut green));
    assert!(green.contains("NOW PASSES"));
    let mut broken = "compile error".to_string();
    assert!(!exec::expect_fail_note(101, &mut broken));
}

#[test]
fn the_release_plan_prints_one_skip_line_per_known_fail() {
    let p = plan::release_with(&root(), false);
    for k in KNOWN_FAIL {
        assert!(p.notes.contains(&known_fail::note(k)), "{}", k.test);
        assert!(known_fail::note(k).starts_with(&format!("SKIPPED (known fail, #{})", k.issue)));
    }
}
