//! Directory suites in asset selection, the catalog check and the heavy filter (#3186).
//! Provenance: Cargo package layout, Rust module paths, and our gate law (AGENTS.md "Testing").

use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::{assets, catalog, plan};

/// The `<crate> heavy: <binary>` step names of a fixture's heavy plan.
fn heavy_names(f: &Fixture) -> Vec<String> {
    plan::heavy_steps(&f.0).into_iter().map(|s| s.name).collect()
}

#[test]
fn an_asset_named_in_a_child_module_selects_its_directory_suite() {
    let f = fixture();
    f.write("crates/app/tests/posting/main.rs", "//! Gate map: covers app::post.\nmod cases;\n");
    f.write(
        "crates/app/tests/posting/cases.rs",
        "#[test]\nfn posts() {\n    let _ = \"assets/posts/fanuc.cps\";\n}\n",
    );
    let changed = vec!["assets/posts/fanuc.cps".to_string()];
    let asset = assets::changed_assets(&changed).pop().expect("one asset");
    let picks = assets::suites_for(&f.0, &asset).expect("asset selection");
    assert_eq!(picks, vec![("app:posting".to_string(), "names fanuc.cps".to_string())]);
}

#[test]
fn a_catalog_change_selects_a_directory_example_catalog() {
    let f = fixture();
    let changed = vec!["examples/catalog/case.camproj".to_string()];
    let (mut whole, mut why) = (BTreeSet::new(), BTreeMap::new());
    catalog::map_catalog(&f.0, &changed, &mut whole, &mut why);
    assert!(whole.is_empty(), "no example_catalog suite: nothing selected, got {whole:?}");
    f.write("crates/app/tests/example_catalog/main.rs", "mod cases;\n");
    catalog::map_catalog(&f.0, &changed, &mut whole, &mut why);
    assert!(whole.contains("app:example_catalog"), "{whole:?}");
}

#[test]
fn a_directory_suite_without_ignored_tests_gets_no_heavy_step() {
    let f = fixture();
    f.write("crates/app/tests/plain/main.rs", "mod sim;\n#[test]\nfn top() {}\n");
    f.write("crates/app/tests/plain/sim.rs", "#[test]\nfn quick() {}\n");
    let names = heavy_names(&f);
    assert!(!names.iter().any(|n| n == "app heavy: plain"), "{names:?}");
    // The fixture's `group` suite holds an `#[ignore]` only in its child module `cases.rs`.
    assert!(names.iter().any(|n| n == "app heavy: group"), "{names:?}");
}

#[test]
fn a_directory_suite_with_a_missing_module_file_keeps_its_heavy_step() {
    let f = fixture();
    f.write("crates/app/tests/gap/main.rs", "#[cfg(feature = \"x\")]\nmod absent;\n#[test]\nfn top() {}\n");
    let names = heavy_names(&f);
    assert!(names.iter().any(|n| n == "app heavy: gap"), "unknown text counts as heavy: {names:?}");
}

#[test]
fn a_flat_suite_keeps_the_mod_line_rule() {
    let f = fixture();
    f.write("crates/app/tests/shared.rs", "mod common;\n#[test]\nfn top() {}\n");
    f.write("crates/app/tests/common/mod.rs", "pub fn helper() {}\n");
    let names = heavy_names(&f);
    assert!(names.iter().any(|n| n == "app heavy: shared"), "{names:?}");
    assert!(!names.iter().any(|n| n == "app heavy: flat"), "{names:?}");
}
