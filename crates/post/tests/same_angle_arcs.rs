//! #3474: a short arc whose written start and written end differ only on
//! the radial line through the written centre (the same angle about it)
//! is no arc block. A control that reads an arc by the angles of its ends
//! sees no sweep there, and an end at the start angle reads as a full turn
//! (a spiral of nearly one turn). The record posts as the straight move to
//! its end, which stays within the output step of the programmed arc.
//!
//! Provenance: our own invariants; the full-circle reading of an arc whose
//! end is at its start (angle) is RS-274 / ISO 6983 circular
//! interpolation. The arc reader is the #3442 one (our own code).
//!
//! Gate map (M4.4): covers assets/posts/carvera.cps,
//! assets/posts/doosan-dnm.cps, assets/posts/fanuc.cps,
//! assets/posts/grbl.cps, assets/posts/grblhal.cps, assets/posts/haas.cps,
//! assets/posts/heidenhain.cps, assets/posts/heidenhain-klartext.cps,
//! assets/posts/linuxcnc.cps, assets/posts/mach4.cps,
//! assets/posts/mazak.cps, assets/posts/sinumerik.cps,
//! assets/posts/tinyg.cps, assets/posts/tormach.cps,
//! camcraft_post::post::arc_guard, camcraft_post::post::engine.

use camcraft_post::post::engine;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[path = "coincident_arcs/reader.rs"]
#[allow(dead_code)]
mod reader;
use reader::arc_blocks;

/// Every shipped mill post (the posts that write XY arcs from a mill stream).
const MILLS: &[&str] = &[
    "carvera",
    "doosan-dnm",
    "fanuc",
    "grbl",
    "grblhal",
    "haas",
    "heidenhain",
    "heidenhain-klartext",
    "linuxcnc",
    "mach4",
    "mazak",
    "sinumerik",
    "tinyg",
    "tormach",
];

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}")).replace("\r\n", "\n")
}

fn machine(stem: &str) -> Value {
    if stem == "heidenhain-klartext" {
        let rel = "assets/machines/fehlmann-picomax55.mch";
        let parsed = camcraft_post::machine::parse_mch(&read(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        return camcraft_post::post::machine::mirror(&parsed);
    }
    json!({"name": "ARC MILL", "spindleMax": 12000, "safeRetractZ": 0})
}

fn post(stem: &str, stream: &str, unit: &str) -> String {
    let cps = read(&format!("assets/posts/{stem}.cps"));
    match engine::run_with(&cps, &machine(stem), stream, &json!({ "Units": unit })) {
        Ok(out) => format!("{}--- warnings: {:?}\n", out.program, out.warnings),
        Err(e) => format!("--- error: {e}\n"),
    }
}

fn units(stem: &str) -> Vec<&'static str> {
    let cps = read(&format!("assets/posts/{stem}.cps"));
    let line = cps.lines().find(|l| l.starts_with("programUnits")).unwrap_or("");
    if line.split('"').nth(1).unwrap_or("").contains("inch") { vec!["mm", "inch"] } else { vec!["mm"] }
}

/// One witness: the start (micrometres), the record's end and its centre.
struct Case {
    unit: &'static str,
    start: (&'static str, &'static str),
    end: (&'static str, &'static str),
    centre: (&'static str, &'static str),
}

/// mm (3 decimals): centre X10 Y5, radius 10.51 um, a CCW sweep of 0.047
/// rad. Written start X10.011 Y5.000, written end X10.010 Y5.000: the same
/// angle about the written centre X10.000 Y5.000.
const MM: Case = Case {
    unit: "mm",
    start: ("10010.51", "5000"),
    end: ("10010.498393", "5000.493788"),
    centre: ("10000", "5000"),
};

/// inch (4 decimals): centre X0.4 Y0.2 in, radius 26.68 um, a CCW sweep
/// of 0.04 rad. Written start X0.4011 Y0.2, written end X0.401 Y0.2.
const INCH: Case = Case {
    unit: "inch",
    start: ("10186.68", "5080"),
    end: ("10186.658660", "5081.066911"),
    centre: ("10160", "5080"),
};

/// A mill section that plunges at the case's start to Z 5.5, cuts the
/// arc (or, for the reference, the straight feed to its end) and retracts.
fn stream(c: &Case, arc: bool) -> String {
    let (sx, sy) = c.start;
    let (ex, ey) = c.end;
    let (cx, cy) = c.centre;
    let cut = if arc {
        format!("Arc CCW X {ex} Y {ey} Z 5500 F 180000 I {cx} J {cy}\n")
    } else {
        format!("Feed X {ex} Y {ey} Z 5500 F 180000\n")
    };
    format!(
        "; HSMW-CNC v1\nPARAM OperationName Same angle\nPARAM SpindleSpeed 6000\nPARAM WorkOffset 1\n\
         PARAM OriginX 0\nPARAM OriginY 0\nPARAM OriginZ 0\nPARAM ToolKind Flat end mill\nPARAM ToolDiameter 4\n\
         PARAM ClearancePlane 18\nPARAM RetractPlane 18\nPARAM TopOfStock 6\nPARAM StockXYLeft -20\nPARAM StockXYRight 40\n\
         PARAM StockXYFront -20\nPARAM StockXYBack 40\nPARAM StockZTop 6\nPARAM StockZBottom -10\n\
         TOOL 1 EM-0400-2FL\nSECTION Same angle\n\
         Rapid X {sx} Y {sy} Z 18000\nPlunge X {sx} Y {sy} Z 5500 F 45000\n{cut}Rapid X {ex} Y {ey} Z 18000\nENDSECTION\n"
    )
}

/// The blocks that the issue asked for: Fanuc and LinuxCNC, mm and inch,
/// post the same-angle arc as one straight move to the written end.
#[test]
fn fanuc_and_linuxcnc_post_a_same_angle_arc_as_a_straight_move() {
    for (stem, case, end) in [
        ("fanuc", &MM, "X10.01 Y5."),
        ("fanuc", &INCH, "X0.401 Y0.2"),
        ("linuxcnc", &MM, "X10.01"),
        ("linuxcnc", &INCH, "X0.401"),
    ] {
        let got = post(stem, &stream(case, true), case.unit);
        assert!(arc_blocks(&got, false).is_empty(), "{stem} {}: an arc block:\n{got}", case.unit);
        assert!(got.ends_with("warnings: []\n"), "{stem} {}: {got}", case.unit);
        assert!(got.lines().any(|l| l.contains("G01") || l.contains("G1 ")), "{stem} {}: no straight move:\n{got}", case.unit);
        assert!(got.contains(end), "{stem} {}: the move ends at {end}:\n{got}", case.unit);
    }
}

/// The lathe witness (mm, diameter X with 3 decimals): a meridian arc
/// about X24 (radius 12) Z-1, radius 10.26 um, a sweep of 0.046 rad.
/// Written start X24.021 Z-1., written end X24.02 Z-1.: the same angle.
fn lathe_stream(arc: bool) -> String {
    let cut = if arc {
        "Arc CW X 12010.249147 Y 0 Z -999.528206 F 300000 I 12000 K -1000\n"
    } else {
        "Feed X 12010.249147 Y 0 Z -999.528206 F 300000\n"
    };
    format!(
        "; HSMW-CNC v1\nPARAM StockXYLeft -20\nPARAM StockXYRight 20\nPARAM StockXYFront -20\nPARAM StockXYBack 20\n\
         PARAM StockZTop 0\nPARAM StockTurnRadius 20\nPARAM ClearancePlane 5\nPARAM SpindleSpeed 1500\nPARAM WorkOffset 1\n\
         TOOL 1 TURN\nSECTION\nRapid X 22000 Y 0 Z 2000\nFeed X 12010.26 Y 0 Z -1000 F 300000\n{cut}\
         Rapid X 22000 Y 0 Z 2000\nENDSECTION\n"
    )
}

/// The table over every shipped mill post and each unit that it offers,
/// and the two lathes: the same-angle arc posts exactly as the straight
/// feed to its end. TinyG writes 4 decimals in mm: there the mm witness
/// is a visible arc (X10.0105 Y5 to X10.0105 Y5.0005) and stays one.
#[test]
fn every_post_writes_a_same_angle_arc_as_its_straight_move() {
    let mut failures = Vec::new();
    for stem in ["fanuc-lathe", "tormach-lathe"] {
        let lathe = json!({"name": "ARC LATHE", "spindleMax": 4000});
        let cps = read(&format!("assets/posts/{stem}.cps"));
        let run = |arc: bool| match engine::run_with(&cps, &lathe, &lathe_stream(arc), &json!({ "Units": "mm" })) {
            Ok(out) => format!("{}--- warnings: {:?}\n", out.program, out.warnings),
            Err(e) => format!("--- error: {e}\n"),
        };
        let (got, want) = (run(true), run(false));
        if !want.lines().any(|l| l.starts_with("G01 X24.02 ") || l.starts_with("G1 X24.02 ")) {
            failures.push(format!("{stem} mm: the reference has no feed move to X24.02\n{want}"));
        }
        if !arc_blocks(&got, true).is_empty() || got != want {
            failures.push(format!("{stem} mm: not the straight move\n--- got\n{got}--- want\n{want}"));
        }
    }
    for &stem in MILLS {
        for unit in units(stem) {
            let case = if unit == "inch" { &INCH } else { &MM };
            let got = post(stem, &stream(case, true), unit);
            let want = post(stem, &stream(case, false), unit);
            if stem == "tinyg" && unit == "mm" {
                if arc_blocks(&got, false).len() != 1 {
                    failures.push(format!("{stem} {unit}: the visible arc is not one arc block\n{got}"));
                }
                continue;
            }
            if !arc_blocks(&got, false).is_empty() {
                failures.push(format!("{stem} {unit}: an arc block\n{got}"));
            } else if got != want {
                failures.push(format!("{stem} {unit}: output differs from the straight move\n--- got\n{got}--- want\n{want}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The rule itself: the written ends differ, but the written end is on the
/// written start's ray from the written centre (or on the centre). A short
/// record is a straight move; a long record (a turn of more than half a
/// circle) stays the arc, and so does a normal short arc.
#[test]
fn arc_fate_reads_a_same_angle_short_arc_as_a_straight_move() {
    let cps = r#"
        function w(k, v) { return k + (Math.round(v * 1000) / 1000).toFixed(3); }
        function fate(s, e, c, ccw) {
          return arcFateOf({ plane: "XY", ccw: ccw, rotary: false,
            start: { X: s[0], Y: s[1], Z: 0 }, end: { X: e[0], Y: e[1], Z: 0 },
            center: { X: c[0], Y: c[1], Z: 0 } }, w);
        }
        function onOpen() {
          writeln("same-angle " + fate([10.01051, 5], [10.010498393, 5.000493788], [10, 5], true));
          writeln("same-angle-cw-is-long " + fate([10.01051, 5], [10.010498393, 5.000493788], [10, 5], false));
          writeln("end-on-centre " + fate([10.0006, 5], [10.00044, 5.00041], [10, 5], true));
          writeln("opposite " + fate([10.0006, 5], [9.9994, 5.0000001], [10, 5], true));
          writeln("normal " + fate([11, 5], [10, 6], [10, 5], true));
        }
    "#;
    let out = engine::run(cps, &Value::Null, "").expect("the rule script runs");
    assert_eq!(
        out.program,
        "same-angle linear\nsame-angle-cw-is-long arc\nend-on-centre linear\nopposite arc\nnormal arc\n"
    );
}
