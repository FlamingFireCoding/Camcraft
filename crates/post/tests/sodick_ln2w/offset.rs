//! #4076: the control-side wire offset of the Sodick LN2W wire post
//! (`sodick-ln2w.cps`) on synthetic streams. Pins the owner's form
//! (`G41` / `G42 Hnnn` on its own line before the lead-in, H from the pass
//! number, `C001` with `H001`, a later pass on the condition that the
//! operation states, `G40` on the lead-out move) and each named refusal of
//! the shared wire-offset law (`wire_offset.rs`).
//! Provenance: synthetic streams from our CNC intermediate contract; the
//! expected words are the statements of the owner of a Sodick AQ600L (LN2W)
//! on #4076, 2026-11-25.
//! Gate map: covers assets/posts/sodick-ln2w.cps, camcraft_post::post::wire_offset,
//! camcraft_post::post::engine, camcraft_post::post::cnc.

use camcraft_post::cl::{ArcPlane, CompSide, MoveKind};
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::engine;
use serde_json::{json, Value};

const SCRIPT: &str = include_str!("../../../../assets/posts/sodick-ln2w.cps");

fn mv(kind: MoveKind, x: f64, y: f64) -> Record {
    Record::Move(MoveRec { kind, x, y, z: 0.0, feed: (kind == MoveKind::Feed).then_some(2.0), center: None, ccw: false, plane: ArcPlane::Xy, abc: None })
}

fn feed(x: f64, y: f64) -> Record {
    mv(MoveKind::Feed, x, y)
}

/// A clockwise arc to `(x, y)` about `(cx, cy)`.
fn arc(x: f64, y: f64, cx: f64, cy: f64) -> Record {
    Record::Move(MoveRec { kind: MoveKind::Feed, x, y, z: 0.0, feed: Some(2.0), center: Some((cx, cy)), ccw: false, plane: ArcPlane::Xy, abc: None })
}

fn param(name: &str, value: Value) -> Record {
    Record::Param { name: name.into(), value }
}

fn comp(side: Option<CompSide>, radius_mm: f64, wear: bool) -> Record {
    Record::Comp { side, radius_mm, wear }
}

fn head(condition: &str) -> Vec<Record> {
    vec![
        param("OperationName", json!("Punch1")),
        param("WorkOffset", json!("G54")),
        param("EdmCondition", json!(condition)),
        Record::Section { begin: true },
        mv(MoveKind::Rapid, -22.0, 10.0),
    ]
}

/// One pass on the control's offset: a 3 mm straight lead-in, one side
/// of the contour, and a straight lead-out with the offset off.
fn pass(side: CompSide, radius: f64, wear: bool) -> Vec<Record> {
    vec![comp(Some(side), radius, wear), feed(-25.0, 10.0), feed(-25.0, -10.0), comp(None, 0.0, wear), feed(-22.0, 10.0)]
}

fn machine(extra: Value) -> Value {
    let mut m = json!({"kind": "wire_edm", "name": "Sodick AQ600L", "edm": {"awt": true, "tank": "submerged", "flushing": "high", "skim_flushing": "low"}});
    if let (Some(e), Some(x)) = (m["edm"].as_object_mut(), extra.as_object()) {
        e.extend(x.clone());
    }
    m
}

fn run(records: &[Record]) -> engine::PostOutput {
    engine::run(SCRIPT, &machine(json!({})), &cnc::serialize(records)).expect("the synthetic stream runs")
}

fn words(out: &engine::PostOutput) -> Vec<&str> {
    out.program.lines().filter(|l| !l.starts_with('(')).collect()
}

fn offset_word(out: &engine::PostOutput) -> bool {
    words(out).iter().any(|l| l.split(' ').any(|w| w == "G41" || w == "G42" || w == "G40" || w.starts_with('H')))
}

fn refused(out: &engine::PostOutput, needles: &[&str]) {
    assert!(
        out.warnings.iter().any(|w| w.contains("Punch1") && needles.iter().all(|n| w.contains(n))),
        "{needles:?} not in {:?}\n{}",
        out.warnings,
        out.program
    );
}

/// The owner's example on #4076, rough cut plus one skim: `C001` with
/// `H001`, the skim on the condition the operation states (`C007`, not a
/// computed `C002`) with `H002`, each `G42 Hnnn` on its own line before the
/// lead-in, and `G40` on each lead-out move. Control and wear compensation
/// write the same words.
#[test]
fn a_control_side_wire_offset_posts_in_the_owners_form() {
    for wear in [false, true] {
        let mut r = head("C001");
        r.extend(pass(CompSide::Right, 0.165, wear));
        r.push(param("EdmCondition", json!("C007")));
        r.extend(pass(CompSide::Right, 0.155, wear));
        r.push(Record::Section { begin: false });
        let out = run(&r);
        assert!(out.warnings.is_empty(), "wear {wear}: {:?}", out.warnings);
        assert_eq!(
            words(&out),
            [
                "G90", "G54", "G00 X-22. Y10. U0. V0.", "Z0.", "T91", "T84", "T94",
                "C001", "G42 H001", "G01 X-25. Y10.", "G01 X-25. Y-10.", "G40 G01 X-22. Y10.",
                "T85", "C007", "G42 H002", "G01 X-25. Y10.", "G01 X-25. Y-10.", "G40 G01 X-22. Y10.",
                "T90", "M02",
            ],
            "wear {wear}:\n{}",
            out.program
        );
        // The operator's note names the register, its condition and the stated value.
        assert!(out.program.contains("H001") && out.program.contains("0.165 MM"), "{}", out.program);
    }
    // The left side is G41.
    let mut r = head("C001");
    r.extend(pass(CompSide::Left, 0.165, false));
    r.push(Record::Section { begin: false });
    let out = run(&r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(words(&out).contains(&"G41 H001"), "{}", out.program);
}

/// The lead-in must be at least the offset radius long (the owner's rule,
/// as for a mill cutter): a shorter one is refused by name, and no offset
/// word or cut is written.
#[test]
fn a_lead_in_shorter_than_the_wire_radius_is_refused_by_name() {
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-22.1, 10.0), feed(-22.1, -10.0), comp(None, 0.0, false), feed(-22.0, 10.0)]);
    r.push(Record::Section { begin: false });
    let out = run(&r);
    refused(&out, &["lead-in", "0.1 mm", "0.165 mm"]);
    assert!(!offset_word(&out) && !words(&out).iter().any(|l| l.starts_with("G01")), "{}", out.program);
    // Exactly the radius is long enough.
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-22.165, 10.0), feed(-22.165, -10.0), comp(None, 0.0, false), feed(-22.0, 10.0)]);
    r.push(Record::Section { begin: false });
    let out = run(&r);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
}

/// The lead-in and the lead-out are straight moves: an arc that would
/// engage or cancel the offset is refused by name.
#[test]
fn an_arc_lead_in_or_lead_out_is_refused_by_name() {
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-25.0, 10.0), arc(-25.0, -10.0, -25.0, 0.0)]);
    r.extend([comp(None, 0.0, false), arc(-22.0, 10.0, -22.0, 0.0), Record::Section { begin: false }]);
    let out = run(&r);
    refused(&out, &["lead-out", "straight"]);
    assert!(!words(&out).iter().any(|l| l.contains("G40")), "{}", out.program);

    let mut r = head("C001");
    r.extend([feed(-23.0, 10.0), comp(Some(CompSide::Right), 0.165, false), arc(-25.0, 10.0, -24.0, 10.0)]);
    r.extend([comp(None, 0.0, false), feed(-22.0, 10.0), Record::Section { begin: false }]);
    let out = run(&r);
    refused(&out, &["lead-in", "straight"]);
    assert!(!offset_word(&out), "{}", out.program);
}

/// H001 always goes with C001 on this control: a first pass on another
/// condition is refused by name, never posted with a register of a
/// different condition.
#[test]
fn a_first_offset_pass_on_a_condition_other_than_c001_is_refused_by_name() {
    let mut r = head("C110");
    r.extend(pass(CompSide::Right, 0.165, false));
    r.push(Record::Section { begin: false });
    let out = run(&r);
    refused(&out, &["H001", "C001", "C110"]);
    assert!(!offset_word(&out), "{}", out.program);
}

/// A section that ends with the offset on would leave G41 / G42 modal for
/// the next cut: refused by name.
#[test]
fn a_section_that_ends_with_the_offset_on_is_refused_by_name() {
    let mut r = head("C001");
    r.extend([comp(Some(CompSide::Right), 0.165, false), feed(-25.0, 10.0), feed(-25.0, -10.0), Record::Section { begin: false }]);
    let out = run(&r);
    refused(&out, &["ends with the wire offset on", "G40"]);
}

/// The C000 approach option cuts the lead-in on C000, but H001 goes with
/// C001: the two together are refused by name.
#[test]
fn the_approach_condition_with_a_wire_offset_is_refused_by_name() {
    let mut r = head("C001");
    r.extend(pass(CompSide::Right, 0.165, false));
    r.push(Record::Section { begin: false });
    let out = engine::run_with(SCRIPT, &machine(json!({})), &cnc::serialize(&r), &json!({"approachCondition": true}));
    let out = out.expect("the synthetic stream runs");
    refused(&out, &["C000", "wire offset"]);
    assert!(!offset_word(&out), "{}", out.program);
}
