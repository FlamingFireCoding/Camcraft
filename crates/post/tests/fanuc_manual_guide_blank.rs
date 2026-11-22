//! #2769: the Fanuc post's opt-in Manual Guide i blank block. With the
//! `manualGuideBlank` option on, a box stock posts `G1902` and a stock
//! cylinder along Z posts `G1900` after the unit block and before the first
//! `G90`; every other stock posts a comment and no block. With the option
//! off, the program is byte-identical to a stream that states no blank, and a
//! script that does not declare `stockBlank = true` never sees the new PARAMs.
//!
//! Provenance: FANUC MANUAL GUIDE i operator's manual (B-63874EN), blank
//! figure section - behaviour only: G1902 B D H (box size) I J K (bottom
//! corner to work origin); G1900 D (diameter) L (length) K (front face to
//! work origin).
//!
//! Gate map (M4.4): covers assets/posts/fanuc.cps,
//! camcraft_post::post::engine, camcraft_post::post::stock_blank.

use camcraft_post::cl::MoveKind;
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::engine;
use serde_json::{json, Value};
use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn param(name: &str, value: Value) -> Record {
    Record::Param { name: name.into(), value }
}

fn mv(kind: MoveKind, x: f64, y: f64, z: f64, feed: Option<f64>) -> Record {
    Record::Move(MoveRec { kind, x, y, z, feed, center: None, ccw: false, plane: cnc_plane(), abc: None })
}

fn cnc_plane() -> camcraft_post::cl::ArcPlane {
    camcraft_post::cl::ArcPlane::Xy
}

/// One face pass over a stock whose XY box is `xy` (left, right, front,
/// back) and whose top is `top`, with the work origin at `origin`. `blank`
/// is the stream's stock-blank PARAMs (none for a legacy stream).
fn stream(origin: [f64; 3], xy: [f64; 4], top: f64, blank: &[(&str, Value)]) -> String {
    let mut recs = vec![
        param("OperationName", json!("Face")),
        param("StartX", json!(origin[0])),
        param("StartY", json!(origin[1])),
        param("StartZ", json!(top + 20.0)),
        param("ToolDiameter", json!(10.0)),
        param("SpindleSpeed", json!(6000.0)),
        param("WorkOffset", json!(1)),
        param("OriginX", json!(origin[0])),
        param("OriginY", json!(origin[1])),
        param("OriginZ", json!(origin[2])),
        param("ClearancePlane", json!(top + 20.0)),
        param("RetractPlane", json!(top + 5.0)),
        param("TopOfStock", json!(top)),
        param("StockXYLeft", json!(xy[0])),
        param("StockXYRight", json!(xy[1])),
        param("StockXYFront", json!(xy[2])),
        param("StockXYBack", json!(xy[3])),
        param("StockZTop", json!(top)),
    ];
    for (n, v) in blank {
        recs.push(param(n, v.clone()));
    }
    recs.extend([
        Record::Tool { number: 1, name: "FACE-10".into() },
        Record::Section { begin: true },
        mv(MoveKind::Rapid, origin[0], origin[1], top + 20.0, None),
        mv(MoveKind::Feed, origin[0], origin[1], top - 1.0, Some(300.0)),
        mv(MoveKind::Rapid, origin[0], origin[1], top + 20.0, None),
        Record::Section { begin: false },
    ]);
    cnc::serialize(&recs)
}

fn post_with(cps: &str, mch: &str, stream: &str, options: &Value) -> String {
    let mch: Value = serde_json::from_str(&read(mch)).expect("machine json");
    let out = engine::run_with(&read(cps), &mch, stream, options).unwrap_or_else(|e| panic!("must post: {e}"));
    assert!(out.warnings.is_empty(), "no warnings: {:?}", out.warnings);
    out.program
}

fn fanuc(stream: &str, options: &Value) -> String {
    post_with("assets/posts/fanuc.cps", "assets/machines/3axis-vmc.mch", stream, options)
}

fn on() -> Value {
    json!({ "manualGuideBlank": true })
}

/// The 100 x 60 x 30 block of the issue, G54 at the top centre.
fn box_stream() -> String {
    stream(
        [50.0, 30.0, 30.0],
        [0.0, 100.0, 0.0, 60.0],
        30.0,
        &[("StockBlank", json!("box")), ("StockBlankZBottom", json!(0.0))],
    )
}

fn index_of(program: &str, prefix: &str) -> usize {
    program
        .lines()
        .position(|l| l.starts_with(prefix))
        .unwrap_or_else(|| panic!("no line starting {prefix:?} in\n{program}"))
}

#[test]
fn a_box_stock_posts_g1902_between_the_unit_block_and_the_first_g90() {
    let nc = fanuc(&box_stream(), &on());
    assert!(nc.lines().any(|l| l == "G1902 B100. D60. H30. I50. J30. K30."), "{nc}");
    let blank = index_of(&nc, "G1902");
    assert!(index_of(&nc, "G21") < blank, "the unit block comes first:\n{nc}");
    assert!(blank < index_of(&nc, "G90"), "the blank stands before G90:\n{nc}");
    assert!(blank < index_of(&nc, "T01"), "the blank stands before the tool change:\n{nc}");
    assert_eq!(nc.matches("G1902").count(), 1, "one blank per program:\n{nc}");
}

#[test]
fn an_offset_origin_and_an_inch_program_keep_the_manuals_signs_and_units() {
    // Work origin 10 mm left of the stock and 5 mm above its top.
    let s = stream(
        [-10.0, 0.0, 35.0],
        [0.0, 100.0, 0.0, 60.0],
        30.0,
        &[("StockBlank", json!("box")), ("StockBlankZBottom", json!(0.0))],
    );
    let nc = fanuc(&s, &on());
    assert!(nc.lines().any(|l| l == "G1902 B100. D60. H30. I-10. J0. K35."), "{nc}");
    let inch = fanuc(&box_stream(), &json!({ "manualGuideBlank": true, "Units": "inch" }));
    assert!(
        inch.lines().any(|l| l == "G1902 B3.937 D2.3622 H1.1811 I1.9685 J1.1811 K1.1811"),
        "{inch}"
    );
}

#[test]
fn a_centred_z_cylinder_posts_g1900() {
    // A 50 mm bar, 40 mm long (Z -40..0), origin on its axis 1 mm below the top face.
    let s = stream(
        [0.0, 0.0, -1.0],
        [-25.0, 25.0, -25.0, 25.0],
        0.0,
        &[("StockBlank", json!("cylinder-z")), ("StockBlankZBottom", json!(-40.0))],
    );
    let nc = fanuc(&s, &on());
    assert!(nc.lines().any(|l| l == "G1900 D50. L40. K1."), "{nc}");
    assert!(index_of(&nc, "G1900") < index_of(&nc, "G90"), "{nc}");
}

#[test]
fn a_stock_the_block_cannot_describe_posts_a_comment_and_no_block() {
    let cases: Vec<(String, &str)> = vec![
        (
            stream([0.0; 3], [0.0, 100.0, 0.0, 60.0], 30.0, &[("StockBlank", json!("from-model"))]),
            "STOCK FROM MODEL",
        ),
        (
            stream([0.0; 3], [0.0, 100.0, 0.0, 60.0], 30.0, &[("StockBlank", json!("rotated-wcs"))]),
            "ROTATED WCS",
        ),
        (
            stream([0.0; 3], [-25.0, 25.0, -25.0, 25.0], 0.0, &[("StockBlank", json!("cylinder-x"))]),
            "CYLINDER ALONG X",
        ),
        // A Z cylinder whose axis is not on the work origin.
        (
            stream(
                [5.0, 0.0, 0.0],
                [-25.0, 25.0, -25.0, 25.0],
                0.0,
                &[("StockBlank", json!("cylinder-z")), ("StockBlankZBottom", json!(-40.0))],
            ),
            "ORIGIN OFF THE BAR AXIS",
        ),
        // A legacy stream: no shape stated at all.
        (stream([0.0; 3], [0.0, 100.0, 0.0, 60.0], 30.0, &[]), "NO STOCK SHAPE"),
    ];
    for (s, why) in cases {
        let nc = fanuc(&s, &on());
        assert!(!nc.contains("G190"), "no blank block for {why}:\n{nc}");
        let note = nc
            .lines()
            .find(|l| l.starts_with("(NO MANUAL GUIDE BLANK"))
            .unwrap_or_else(|| panic!("a comment says why ({why}):\n{nc}"));
        assert!(note.contains(why), "{note} names {why}");
        assert!(note.len() <= 60, "{note} fits the comment budget");
    }
}

#[test]
fn the_option_off_posts_the_same_program_as_a_stream_with_no_blank() {
    let plain = stream([50.0, 30.0, 30.0], [0.0, 100.0, 0.0, 60.0], 30.0, &[]);
    assert_eq!(fanuc(&box_stream(), &Value::Null), fanuc(&plain, &Value::Null));
}

#[test]
fn a_script_without_the_manifest_flag_never_sees_the_blank_params() {
    // The Klartext post refuses every PARAM it does not list (a positive
    // list), so it would refuse the stream if the engine passed them on.
    let fixture = read("crates/post/tests/klartext/contour.cnc");
    let with_blank = fixture.replacen(
        "PARAM Coolant off",
        "PARAM StockBlank box\nPARAM StockBlankZBottom 3\nPARAM Coolant off",
        1,
    );
    assert_ne!(fixture, with_blank, "the fixture changed");
    let a = post_with(
        "assets/posts/heidenhain-klartext.cps",
        "assets/machines/fehlmann-picomax55.mch",
        &fixture,
        &Value::Null,
    );
    let b = post_with(
        "assets/posts/heidenhain-klartext.cps",
        "assets/machines/fehlmann-picomax55.mch",
        &with_blank,
        &Value::Null,
    );
    assert_eq!(a, b);
}
