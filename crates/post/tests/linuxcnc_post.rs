//! Issue #4151: LinuxCNC post header clears G92 offsets and states G97.
//!
//! Provenance: public LinuxCNC G-code reference (linuxcnc.org/docs)
//! for G92.1 (reset G92 offsets) and G97 (constant spindle RPM mode);
//! no vendor materials consulted. Original clean-room tests.
//!
//! Gate map: covers assets/posts/linuxcnc.cps, camcraft_post::post::engine.

use camcraft_post::cl::MoveKind;
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::engine;
use serde_json::json;
use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn simple_stream() -> String {
    cnc::serialize(&[
        Record::Param {
            name: "OperationName".into(),
            value: json!("Face1"),
        },
        Record::Param {
            name: "WorkOffset".into(),
            value: json!(1),
        },
        Record::Param {
            name: "ClearancePlane".into(),
            value: json!(25.0),
        },
        Record::Param {
            name: "RetractPlane".into(),
            value: json!(10.0),
        },
        Record::Param {
            name: "SpindleSpeed".into(),
            value: json!(3000.0),
        },
        Record::Tool {
            number: 1,
            name: "10mm Endmill".into(),
        },
        Record::Section { begin: true },
        Record::Move(MoveRec {
            kind: MoveKind::Rapid,
            x: 0.0,
            y: 0.0,
            z: 10.0,
            feed: None,
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        }),
        Record::Move(MoveRec {
            kind: MoveKind::Feed,
            x: 10.0,
            y: 0.0,
            z: -1.0,
            feed: Some(500.0),
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        }),
        Record::Section { begin: false },
    ])
}

#[test]
fn header_clears_g92_and_states_g97_by_default() {
    let cps = read("assets/posts/linuxcnc.cps");
    let mch = json!({ "name": "LinuxCNC Mill", "safeRetractZ": 50.0 });
    let stream = simple_stream();

    let out = engine::run(&cps, &mch, &stream).expect("post run");
    let nc = &out.program;

    // Must emit G92.1 and G97 in the header modal safety block
    let header_line = nc
        .lines()
        .find(|l| l.contains("G21") && l.contains("G17"))
        .expect("modal header block");

    assert!(
        header_line.contains("G92.1"),
        "header must clear G92 offsets: {header_line}"
    );
    assert!(
        header_line.contains("G97"),
        "header must state G97 constant RPM mode: {header_line}"
    );

    // G92.1 and G97 must appear before the first spindle start S3000 M3
    let s_pos = nc.find("S3000 M3").expect("spindle start block");
    let g92_pos = nc.find("G92.1").expect("G92.1 in nc");
    let g97_pos = nc.find("G97").expect("G97 in nc");

    assert!(g92_pos < s_pos, "G92.1 must precede spindle start");
    assert!(g97_pos < s_pos, "G97 must precede spindle start");
}

#[test]
fn header_omits_g92_1_when_clearg92_option_is_false() {
    let cps = read("assets/posts/linuxcnc.cps");
    let mch = json!({ "name": "LinuxCNC Mill", "safeRetractZ": 50.0 });
    let stream = simple_stream();
    let options = json!({ "clearG92": false });

    let out = engine::run_with(&cps, &mch, &stream, &options).expect("post run");
    let nc = &out.program;

    let header_line = nc
        .lines()
        .find(|l| l.contains("G21") && l.contains("G17"))
        .expect("modal header block");

    assert!(
        !header_line.contains("G92.1"),
        "G92.1 must be omitted when clearG92 is false: {header_line}"
    );
    assert!(
        header_line.contains("G97"),
        "G97 must still be emitted: {header_line}"
    );
}

#[test]
fn header_includes_g92_1_when_clearg92_option_is_true() {
    let cps = read("assets/posts/linuxcnc.cps");
    let mch = json!({ "name": "LinuxCNC Mill", "safeRetractZ": 50.0 });
    let stream = simple_stream();
    let options = json!({ "clearG92": true });

    let out = engine::run_with(&cps, &mch, &stream, &options).expect("post run");
    let nc = &out.program;

    let header_line = nc
        .lines()
        .find(|l| l.contains("G21") && l.contains("G17"))
        .expect("modal header block");

    assert!(
        header_line.contains("G92.1"),
        "G92.1 must be emitted when clearG92 is true: {header_line}"
    );
    assert!(
        header_line.contains("G97"),
        "G97 must be emitted: {header_line}"
    );
}
