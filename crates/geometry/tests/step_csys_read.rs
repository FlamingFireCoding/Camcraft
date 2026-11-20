//! #2789: the tool coordinate-system reader on solidcore's Part 21 parser
//! keeps the file-level refusals of the ruststep reader it replaced, and
//! names the readings that changed: an entity id defined twice is refused,
//! several DATA sections are read as one, a missing DATA section is refused
//! by its old text, and no corrupt input panics or hangs. The fixture
//! results themselves are pinned end to end in the app's `tool_csys` suite.
//! Provenance: our own test design; Part 21 structure per ISO 10303-21.
//!
//! Gate map (#2789): covers camcraft_geometry::step_csys.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use camcraft_geometry::step_csys::{read_tool_csys, CsysRole};

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name);
    std::fs::read_to_string(path).expect("fixture reads").replace("\r\n", "\n")
}

/// `tool_csys_iso13399.step` with `from` replaced by `to` (which must be
/// in it).
fn edited(from: &str, to: &str) -> String {
    let text = fixture("tool_csys_iso13399.step");
    assert!(text.contains(from), "the fixture has {from:?}");
    text.replacen(from, to, 1)
}

#[test]
fn the_fixture_reads_through_the_part_21_parser() {
    let r = read_tool_csys(&fixture("tool_csys_iso13399.step")).expect("reads");
    let cip = r.find(CsysRole::InProcess).expect("one CIP");
    assert_eq!(cip.entity, 50);
    assert!((cip.origin[2] + 75.0).abs() < 1e-12, "{:?}", cip.origin);
    assert_eq!(r.find(CsysRole::Primary).map(|c| c.entity), Ok(30));
}

#[test]
fn an_entity_id_defined_twice_is_refused_by_name() {
    // A second #51 that moves the CIP: the old reader took the last
    // definition, the parser keeps the first. Neither is safe to use.
    let text = edited(
        "#51 = CARTESIAN_POINT('', (0.0, 0.0, -75.0));",
        "#51 = CARTESIAN_POINT('', (0.0, 0.0, -75.0));\n#51 = CARTESIAN_POINT('', (0.0, 0.0, -90.0));",
    );
    let e = read_tool_csys(&text).expect_err("a duplicate id is refused");
    assert!(e.contains("entity #51 is defined more than once"), "{e}");
}

#[test]
fn a_file_with_no_data_section_is_refused_by_its_old_text() {
    let text = "ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('X'));\nENDSEC;\nEND-ISO-10303-21;\n";
    assert_eq!(read_tool_csys(text).err().as_deref(), Some("file has no DATA section"));
    // An empty DATA section is a file that names no coordinate systems.
    let empty = "ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\nENDSEC;\nEND-ISO-10303-21;\n";
    let r = read_tool_csys(empty).expect("an empty DATA section reads");
    assert_eq!(r.none_found().as_deref(), Some("the file names no coordinate systems"));
}

#[test]
fn several_data_sections_are_read_as_one() {
    // The CIP and its points move to a second DATA section.
    let text = fixture("tool_csys_iso13399.step");
    let cip = "#50 = AXIS2_PLACEMENT_3D('CIP', #51, #52, #53);\n";
    let text = text.replacen(cip, "", 1).replacen(
        "ENDSEC;\nEND-ISO-10303-21;",
        &format!("ENDSEC;\nDATA;\n{cip}ENDSEC;\nEND-ISO-10303-21;"),
        1,
    );
    let r = read_tool_csys(&text).expect("reads");
    assert_eq!(r.find(CsysRole::InProcess).map(|c| c.entity), Ok(50));
}

#[test]
fn a_syntax_error_names_the_entity_and_its_line() {
    let text = edited("#41 = CARTESIAN_POINT('', (0.0, 0.0, -40.0));", "#41 = CARTESIAN_POINT('', (0.0,, -40.0));");
    let e = read_tool_csys(&text).expect_err("refused");
    assert!(e.contains("entity #41") && e.contains("line 20"), "{e}");
}

/// The parser's own depth cap (64) does not read an empty innermost `()`
/// one level past it, so it takes this record. That is why the app's
/// nesting scan (`step_depth`, which refuses it) stays in front of every
/// read: the app's `step_nesting` suite pins the refusal.
#[test]
fn the_parser_alone_takes_an_empty_list_one_past_its_depth_cap() {
    let deep = format!("#999999 = DEEP_LIST({}{});\n", "(".repeat(64), ")".repeat(64));
    let text = edited("ENDSEC;\nEND-ISO-10303-21;", &format!("{deep}ENDSEC;\nEND-ISO-10303-21;"));
    assert!(read_tool_csys(&text).is_ok(), "the parser has no law for this record");
}

#[test]
fn corrupt_input_is_a_clean_error_never_a_panic() {
    let good = fixture("tool_csys_iso13399.step");
    let mut cases: Vec<String> = vec![
        String::new(),
        "ISO-10303-21; junk".into(),
        "ISO-10303-21;\nDATA;\n#1 = A('open);\nENDSEC;\nEND-ISO-10303-21;\n".into(),
        "ISO-10303-21;\nDATA;\n#1 = A(1);\n/* never closed\nENDSEC;\nEND-ISO-10303-21;\n".into(),
        "ISO-10303-21;\nDATA;\n#1 = A(\u{0});\nENDSEC;\nEND-ISO-10303-21;\n".into(),
        format!("ISO-10303-21;\nDATA;\n#1 = A({});\nENDSEC;\nEND-ISO-10303-21;\n", "(".repeat(5000)),
    ];
    // Every prefix of the fixture at a coarse step: truncated files.
    let step = (good.len() / 40).max(1);
    cases.extend((0..good.len()).step_by(step).filter_map(|n| good.get(..n)).map(str::to_string));
    for text in &cases {
        let _ = read_tool_csys(text);
    }
    // A self-referencing placement and a unit that loops refuse, never hang.
    let looped = edited("#50 = AXIS2_PLACEMENT_3D('CIP', #51, #52, #53);", "#50 = AXIS2_PLACEMENT_3D('CIP', #50, #50, #50);");
    let r = read_tool_csys(&looped).expect("reads");
    assert!(r.find(CsysRole::InProcess).is_err_and(|e| e.contains("#50")), "{:?}", r.refused);
}
