//! #4045: the Sodick LN2W wire post (`sodick-ln2w.cps`) on synthetic
//! streams. Pins the start sequence that the owner of an AQ600L gave, the
//! absence of a unit word (the control alarms on G21), the taper words
//! (TP / TN, G51 / G52 A, G50), the skim flushing and each named refusal.
//! Provenance: synthetic streams from our CNC intermediate contract; the
//! expected words are the owner's statements and the LN manual facts that
//! docs/decisions/4045-sodick-ln2w-dialect.md records.
//! Gate map: covers assets/posts/sodick-ln2w.cps, camcraft_post::post::engine,
//! camcraft_post::post::cnc.

use camcraft_post::cl::{ArcPlane, CompSide, MoveKind};
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::engine;
use serde_json::{json, Value};

const SCRIPT: &str = include_str!("../../../../assets/posts/sodick-ln2w.cps");

fn mv(kind: MoveKind, x: f64, y: f64) -> Record {
    Record::Move(MoveRec { kind, x, y, z: 0.0, feed: (kind == MoveKind::Feed).then_some(2.0), center: None, ccw: false, plane: ArcPlane::Xy, abc: None })
}

fn param(name: &str, value: Value) -> Record {
    Record::Param { name: name.into(), value }
}

/// The section's opening params; `wire` holds the taper params.
fn head(wire: &[(&str, Value)]) -> Vec<Record> {
    let mut r = vec![param("OperationName", json!("Punch1")), param("WorkOffset", json!("G54")), param("EdmCondition", json!("C001"))];
    r.extend(wire.iter().map(|(n, v)| param(n, v.clone())));
    r.extend([Record::Section { begin: true }, mv(MoveKind::Rapid, -22.0, 10.0)]);
    r
}

fn machine(extra: Value) -> Value {
    let mut m = json!({"kind": "wire_edm", "name": "Sodick AQ600L", "edm": {"awt": true, "tank": "submerged", "flushing": "high", "skim_flushing": "low"}});
    if let (Some(e), Some(x)) = (m["edm"].as_object_mut(), extra.as_object()) {
        e.extend(x.clone());
    }
    m
}

fn run(machine: &Value, records: &[Record]) -> engine::PostOutput {
    engine::run(SCRIPT, machine, &cnc::serialize(records)).expect("the synthetic stream runs")
}

/// The program's executable lines (no comment lines).
fn words(out: &engine::PostOutput) -> Vec<&str> {
    out.program.lines().filter(|l| !l.starts_with('(')).collect()
}

/// A 3 degree taper: the lead-in leans, one side, the lead-out stands upright.
fn tapered(angle: f64) -> Vec<Record> {
    let mut r = head(&[
        ("WireTaper", json!(3.0)),
        ("WireTaperDialect", json!("angle")),
        ("WireCorner", json!("mitre")),
        ("WirePlane", json!(12.0)),
        ("WireBottom", json!(0.0)),
    ]);
    r.extend([
        param("WireAngle", json!(angle)),
        mv(MoveKind::Feed, -25.0, 10.0),
        param("WireAngle", json!(angle)),
        mv(MoveKind::Feed, -25.0, -10.0),
        param("WireAngle", json!(0.0)),
        mv(MoveKind::Feed, -22.0, 10.0),
        Record::Section { begin: false },
    ]);
    r
}

#[test]
fn the_start_sequence_is_the_owners_and_no_unit_word_is_written() {
    let mut r = head(&[]);
    r.extend([mv(MoveKind::Feed, -25.838, 10.0), mv(MoveKind::Feed, -22.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_eq!(
        words(&out),
        ["G90", "G54", "G00 X-22. Y10.", "Z0.", "T91", "T84", "T94", "C001", "U0. V0.", "G01 X-25.838 Y10.", "G01 X-22. Y10.", "T90", "M02"],
        "{}",
        out.program
    );
    for banned in ["G21", "G20", "G40", "G92", "M30"] {
        assert!(!words(&out).iter().any(|l| l.split(' ').any(|w| w == banned)), "{banned} in\n{}", out.program);
    }
    assert!(out.program.contains("(SODICK LN2W WORDS - NOT VERIFIED ON A CONTROL - CHECK BEFORE CUTTING)"), "{}", out.program);
    assert!(SCRIPT.contains("programUnits = \"mm\";"), "an inch document must be refused: no unit word can be written");
}

#[test]
fn a_taper_posts_tp_tn_and_g51_g52_a_with_g50_before_the_upright_move() {
    let m = machine(json!({"table_to_part_bottom": 5.0}));
    let out = run(&m, &tapered(3.0));
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert_eq!(
        words(&out),
        [
            "G90", "G54", "TP17.", "TN5.", "G00 X-22. Y10.", "Z0.", "T91", "T84", "T94", "C001", "U0. V0.", "G51 A3.", "G01 X-25. Y10.",
            "G01 X-25. Y-10.", "G50", "G01 X-22. Y10.", "T90", "M02"
        ],
        "{}",
        out.program
    );
    // A negative stream angle leans the upper guide right of travel.
    let right = run(&m, &tapered(-3.0));
    assert!(right.warnings.is_empty(), "{:?}", right.warnings);
    assert!(words(&right).contains(&"G52 A3.") && !words(&right).iter().any(|l| l.contains("G51")), "{}", right.program);
}

#[test]
fn a_taper_without_its_heights_or_its_dialect_is_refused_by_name() {
    let cut = |out: &engine::PostOutput| words(out).iter().any(|l| l.starts_with("G01") || *l == "T91");
    let out = run(&machine(json!({})), &tapered(3.0));
    assert!(out.warnings.iter().any(|w| w.contains("Punch1") && w.contains("edm.table_to_part_bottom")), "{:?}", out.warnings);
    assert!(!cut(&out), "{}", out.program);
    // A guide-offset (U V) stream: the machine file states no edm.taper_angle.
    let mut uv = head(&[("WireTaper", json!(3.0))]);
    uv.extend([param("WireU", json!(0.6)), param("WireV", json!(0.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({"table_to_part_bottom": 5.0})), &uv);
    assert!(out.warnings.iter().any(|w| w.contains("states no edm.taper_angle")), "{:?}", out.warnings);
    assert!(!cut(&out), "{}", out.program);
    // A 4-axis cut.
    let mut four = head(&[("WireFourAxis", json!("arc length"))]);
    four.extend([param("WireU", json!(0.6)), param("WireV", json!(0.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({"table_to_part_bottom": 5.0})), &four);
    assert!(out.warnings.iter().any(|w| w.contains("is a 4-axis cut")), "{:?}", out.warnings);
    assert!(!cut(&out), "{}", out.program);
    // An angle word in a vertical section, and a lean that is never stood upright.
    let mut stray = head(&[]);
    stray.extend([param("WireAngle", json!(3.0)), mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &stray);
    assert!(out.warnings.iter().any(|w| w.contains("states no angle-word taper")), "{:?}", out.warnings);
    assert!(!words(&out).iter().any(|l| l.starts_with("G01")), "{}", out.program);
    let mut leaning = tapered(3.0);
    leaning.retain(|r| !matches!(r, Record::Param { name, value } if name == "WireAngle" && value == &json!(0.0)));
    let out = run(&machine(json!({"table_to_part_bottom": 5.0})), &leaning);
    assert!(out.warnings.iter().any(|w| w.contains("no angle word while the wire leans")), "{:?}", out.warnings);
}

#[test]
fn a_skim_pass_takes_the_low_flushing_word_and_its_own_condition() {
    let mut r = head(&[]);
    r.extend([
        mv(MoveKind::Feed, -25.0, 10.0),
        mv(MoveKind::Feed, -22.0, 10.0),
        param("EdmCondition", json!("C002")),
        mv(MoveKind::Feed, -25.02, 10.0),
        mv(MoveKind::Feed, -22.0, 10.0),
        Record::Section { begin: false },
    ]);
    let out = run(&machine(json!({})), &r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let w = words(&out);
    let at = w.iter().position(|l| *l == "T85").expect("the skim's flushing word");
    assert_eq!(&w[at..at + 3], ["T85", "C002", "G01 X-25.02 Y10."], "{}", out.program);
    assert_eq!(w.iter().filter(|l| **l == "T91").count(), 1, "one threading for the section: {}", out.program);
}

#[test]
fn a_control_side_wire_offset_is_refused_by_name() {
    let mut r = head(&[]);
    r.insert(r.len() - 1, Record::Comp { side: Some(CompSide::Left), radius_mm: 0.165, wear: false });
    r.extend([mv(MoveKind::Feed, -25.0, 10.0), Record::Section { begin: false }]);
    let out = run(&machine(json!({})), &r);
    assert!(out.warnings.iter().any(|w| w.contains("G41/G42 H") && w.contains("not supported")), "{:?}", out.warnings);
    assert!(!words(&out).iter().any(|l| l.starts_with("G01") || l.contains("G41") || l.contains("G42")), "{}", out.program);
}
