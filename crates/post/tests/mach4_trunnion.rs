//! Provenance: clean-room 3+2 XYZAC trunnion, rollover, clamp, RTCP guard,
//! and simultaneous 5-axis refusal tests for Mach4 Mill (issue #1968).
//!
//! Gate map (M4.4): covers assets/posts/mach4.cps, post::kinematics, post::machine.

use camcraft_post::kinematics::{
    forward_part, AcTrunnionConfig, MachinePose, RotaryAxisConfig, RotarySignConvention,
};
use camcraft_post::machine::parse_mch;
use camcraft_post::post::{engine, machine};
use serde_json::{json, Value};
use std::f64::consts::PI;
use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn mach4_post() -> String {
    read("assets/posts/mach4.cps")
}

fn default_vmc() -> Value {
    serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("vmc machine json")
}

struct TrunnionSpec<'a> {
    unit: &'a str,
    rtcp: Option<bool>,
    sign_conv: &'a str,
    a_zero: f64,
    c_zero: f64,
    c_rollover: bool,
    pivot: [f64; 3],
    offset: [f64; 3],
}

fn trunnion_mch_json(spec: &TrunnionSpec<'_>) -> String {
    let mut controller = json!({
        "synced_configuration": { "parts": {
            "X": { "coordinate": 0, "max_normal_speed": 4000.0, "max_rapid_speed": 8000.0 },
            "Y": { "coordinate": 1, "max_normal_speed": 4000.0, "max_rapid_speed": 8000.0 },
            "Z": { "coordinate": 2, "max_normal_speed": 3000.0, "max_rapid_speed": 6000.0 },
            "A": { "coordinate": 3, "max_normal_speed": 1500.0, "max_rapid_speed": 3000.0 },
            "C": { "coordinate": 4, "max_normal_speed": 2000.0, "max_rapid_speed": 4000.0 }
        } }
    });
    if let Some(flag) = spec.rtcp {
        controller["rtcp"] = json!(flag);
    }
    json!({
        "general": { "model": "MACH4 AC TRUNNION", "vendor": "Camcraft", "kind": "mill" },
        "kinematics": { "default": {
            "units": { "length": spec.unit, "angle": "degrees" },
            "sign_convention": spec.sign_conv,
            "parts": [{ "id": "base", "name": "table", "type": "table", "parts": [
                {
                    "id": "a_axis", "name": "A", "type": "rotary", "axis": [1.0, 0.0, 0.0],
                    "pivot": spec.pivot, "travel": { "min": -110.0, "max": 30.0 },
                    "zero_offset": spec.a_zero, "rollover": false,
                    "parts": [{
                        "id": "c_axis", "name": "C", "type": "rotary", "axis": [0.0, 0.0, 1.0],
                        "offset": spec.offset, "travel": { "min": -720.0, "max": 720.0 },
                        "zero_offset": spec.c_zero, "rollover": spec.c_rollover
                    }]
                },
                { "id": "x", "name": "X", "type": "linear", "travel": { "min": -300.0, "max": 300.0 },
                  "parts": [{ "id": "y", "name": "Y", "type": "linear", "travel": { "min": -200.0, "max": 200.0 } }] },
                { "id": "z", "name": "Z", "type": "linear", "travel": { "min": -250.0, "max": 0.0 }, "safe_retract": 0.0,
                  "parts": [{ "id": "head", "name": "head", "type": "head",
                    "spindle": { "min_speed": 1000.0, "max_speed": 18000.0 },
                    "tool_station": { "coolants": [{ "name": "Flood", "mCode": "M8" }, { "name": "Mist", "mCode": "M7" }] }
                  }] }
            ] }]
        } },
        "controller": controller,
        "tooling": { "number_of_tools": 16 }
    })
    .to_string()
}

fn static_rotaries_mch(name: &str, rotaries: Value) -> Value {
    json!({
        "name": name,
        "safeRetractZ": 0.0,
        "rtcp": false,
        "kinematics": { "static": { "rotaries": rotaries } }
    })
}

fn stream_axis_from_abc(a_deg: f64, b_deg: f64, c_deg: f64) -> [f64; 3] {
    let (sa, ca) = (a_deg * PI / 180.0).sin_cos();
    let (sb, cb) = (b_deg * PI / 180.0).sin_cos();
    let (sc, cc) = (c_deg * PI / 180.0).sin_cos();
    [ca * sb * cc + sa * sc, ca * sb * sc - sa * cc, ca * cb]
}

fn norm3(v: [f64; 3]) -> f64 { (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() }
fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }

fn word_value(line: &str, letter: char) -> Option<f64> {
    let line = line.split('(').next().unwrap_or("");
    for tok in line.split_whitespace() {
        if let Some(rest) = tok.strip_prefix(letter)
            && let Ok(v) = rest.parse::<f64>()
        {
            return Some(v);
        }
    }
    None
}

#[test]
fn mach4_3plus2_indexed_ac_trunnion_round_trips_kinematics() {
    let cps = mach4_post();
    let pivot = [120.0, -40.0, -150.0];
    let offset = [0.0, 12.5, 28.0];
    let a_zero = 10.0;
    let c_zero = -45.0;

    let stream = "\
PARAM WorkOffset G54
PARAM ClearancePlane 50
PARAM RetractPlane 10
PARAM OriginX 0
PARAM OriginY 0
PARAM OriginZ 0
TOOL 2 \"8MM BALL\"
PARAM SpindleSpeed 9000
SECTION VerticalTop
Rapid X 10000 Y 5000 Z 15000
Feed X 10000 Y 5000 Z 0 F 400000
Feed X 20000 Y 5000 Z 0 F 400000
ENDSECTION
SECTION IndexedSide45
PARAM ClearancePlane 50
PARAM RetractPlane 10
Rapid X 0 Y 20000 Z 30000 A -45000 B 0 C 0
Feed X 10000 Y 20000 Z 25000 F 500000 A -45000 B 0 C 0
Feed X 20000 Y 20000 Z 25000 F 500000 A -45000 B 0 C 0
ENDSECTION
SECTION IndexedRight90Arc
PARAM ClearancePlane 50
PARAM RetractPlane 10
Rapid X 15000 Y 10000 Z 5000 A 0 B -90000 C 0
Feed X 15000 Y 0 Z 5000 F 500000 A 0 B -90000 C 0
Arc CW X 15000 Y 0 Z 15000 F 500000 A 0 B -90000 C 0 J 0 K 10000
ENDSECTION
SECTION BackToVertical
PARAM ClearancePlane 50
PARAM RetractPlane 10
Rapid X 0 Y 0 Z 20000
Feed X 5000 Y 5000 Z 0 F 300000
ENDSECTION
";

    for (sign, unit) in [("table_rh", "mm"), ("tool_rh", "mm"), ("table_rh", "inch")] {
        let scale = if unit == "inch" { 25.4 } else { 1.0 };
        let spec = TrunnionSpec {
            unit,
            rtcp: Some(false),
            sign_conv: sign,
            a_zero,
            c_zero,
            c_rollover: false,
            pivot: [pivot[0] / scale, pivot[1] / scale, pivot[2] / scale],
            offset: [offset[0] / scale, offset[1] / scale, offset[2] / scale],
        };
        let parsed = parse_mch(&trunnion_mch_json(&spec)).expect("trunnion parses");
        let mch = machine::mirror(&parsed);
        let opts = if unit == "inch" { json!({ "Units": "inch" }) } else { Value::Null };
        let out = engine::run_with(&cps, &mch, stream, &opts).expect("3+2 runs");
        assert!(out.warnings.is_empty(), "{sign}/{unit} warnings: {:?}", out.warnings);
        let p = &out.program;
        assert!(p.contains(if unit == "inch" { "G20" } else { "G21" }), "unit code missing:\n{p}");
        assert!(!p.contains("G43.4") && !p.contains("G93"), "3+2 Mach4 must use G94 without G43.4/G93:\n{p}");

        let cfg = AcTrunnionConfig {
            a: RotaryAxisConfig {
                name: "A".into(),
                axis: [1.0, 0.0, 0.0],
                travel: Some((-110.0, 30.0)),
                zero_offset_deg: a_zero,
                rollover: false,
            },
            c: RotaryAxisConfig {
                name: "C".into(),
                axis: [0.0, 0.0, 1.0],
                travel: Some((-720.0, 720.0)),
                zero_offset_deg: c_zero,
                rollover: false,
            },
            spindle_axis: [0.0, 0.0, 1.0],
            a_pivot: pivot,
            c_offset: offset,
            wcs_offset: [0.0, 0.0, 0.0],
            work_datum: [pivot[0] + offset[0], pivot[1] + offset[1], pivot[2] + offset[2]],
            sign: if sign == "table_rh" { RotarySignConvention::TableRightHand } else { RotarySignConvention::ToolRightHand },
            pole_sin_tol: 1e-6,
            weight_a: 1.0,
            weight_c: 1.0,
            initial_c_deg: 0.0,
        };

        let (mut cur_a, mut cur_c) = (a_zero, c_zero);
        let (mut cur_x, mut cur_y, mut cur_z) = (0.0, 0.0, 0.0);
        let mut feed_points = Vec::new();
        for line in p.lines() {
            if let Some(a) = word_value(line, 'A') { cur_a = a; }
            if let Some(c) = word_value(line, 'C') { cur_c = c; }
            let has_xyz = word_value(line, 'X').is_some() || word_value(line, 'Y').is_some() || word_value(line, 'Z').is_some();
            if let Some(x) = word_value(line, 'X') { cur_x = x * scale; }
            if let Some(y) = word_value(line, 'Y') { cur_y = y * scale; }
            if let Some(z) = word_value(line, 'Z') { cur_z = z * scale; }
            if has_xyz && (line.starts_with("G01 ") || line.starts_with("G02 ") || line.starts_with("G03 ")) {
                feed_points.push(([cur_x, cur_y, cur_z], cur_a, cur_c));
            }
        }

        let expected = [
            ([10.0, 5.0, 0.0], [0.0, 0.0, 1.0]),
            ([20.0, 5.0, 0.0], [0.0, 0.0, 1.0]),
            ([10.0, 20.0, 25.0], stream_axis_from_abc(-45.0, 0.0, 0.0)),
            ([20.0, 20.0, 25.0], stream_axis_from_abc(-45.0, 0.0, 0.0)),
            ([15.0, 0.0, 5.0], stream_axis_from_abc(0.0, -90.0, 0.0)),
            ([15.0, 0.0, 15.0], stream_axis_from_abc(0.0, -90.0, 0.0)),
            ([5.0, 5.0, 0.0], [0.0, 0.0, 1.0]),
        ];
        assert_eq!(feed_points.len(), expected.len(), "feed count in {sign}/{unit}:\n{p}");
        let tip_tol = if unit == "inch" { 5e-3 } else { 2e-3 };
        for (i, ((xyz, a, c), (want_tip, want_axis))) in feed_points.iter().zip(expected.iter()).enumerate() {
            let rec = forward_part(&cfg, MachinePose { xyz: *xyz, a_deg: *a, c_deg: *c }).expect("forward_part");
            assert!(norm3(sub3(rec.tip, *want_tip)) <= tip_tol, "{sign}/{unit} pt {i}: {:?} vs {:?}", rec.tip, want_tip);
            assert!(dot3(rec.axis, *want_axis) > 1.0 - 1e-6, "{sign}/{unit} axis {i}: {:?} vs {:?}", rec.axis, want_axis);
        }
        assert!(p.contains("G02 ") || p.contains("G03 "), "rotated YZ arc missing:\n{p}");
    }
}

#[test]
fn mach4_rotary_rollover_unwrapped_vs_360_and_clamp_order() {
    let cps = mach4_post();
    let spec = TrunnionSpec {
        unit: "mm",
        rtcp: Some(false),
        sign_conv: "table_rh",
        a_zero: 0.0,
        c_zero: 0.0,
        c_rollover: true,
        pivot: [0.0, 0.0, 0.0],
        offset: [0.0, 0.0, 0.0],
    };
    let mch = machine::mirror(&parse_mch(&trunnion_mch_json(&spec)).expect("parses"));
    let stream = "\
PARAM WorkOffset G54
PARAM ClearancePlane 30
TOOL 1 \"6MM EM\"
PARAM SpindleSpeed 8000
SECTION S1
Rapid X 10000 Y 0 Z 10000 A -30000 B 0 C 20000
Feed X 10000 Y 0 Z 0 F 400000 A -30000 B 0 C 20000
ENDSECTION
SECTION S2
PARAM ClearancePlane 30
Rapid X 10000 Y 0 Z 10000 A -30000 B 0 C 140000
Feed X 10000 Y 0 Z 0 F 400000 A -30000 B 0 C 140000
ENDSECTION
SECTION S3
PARAM ClearancePlane 30
Rapid X 10000 Y 0 Z 10000 A -30000 B 0 C 260000
Feed X 10000 Y 0 Z 0 F 400000 A -30000 B 0 C 260000
ENDSECTION
SECTION S4
PARAM ClearancePlane 30
Rapid X 10000 Y 0 Z 10000 A -30000 B 0 C 20000
Feed X 10000 Y 0 Z 0 F 400000 A -30000 B 0 C 20000
ENDSECTION
";
    let out_360 = engine::run_with(&cps, &mch, stream, &json!({ "rolloverMode": "rollover_360" })).expect("rollover_360");
    assert!(out_360.warnings.is_empty(), "{:?}", out_360.warnings);
    let c_vals: Vec<f64> = out_360.program.lines().filter_map(|l| word_value(l, 'C')).collect();
    assert!(!c_vals.is_empty() && c_vals.iter().all(|c| (0.0..360.0).contains(c)), "C out of [0, 360): {c_vals:?}");

    let lines: Vec<&str> = out_360.program.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        if line.contains("G00 ") && (word_value(line, 'A').is_some() || word_value(line, 'C').is_some()) {
            let prev = lines[idx.saturating_sub(1)];
            assert!(prev.contains("G28") || prev.contains("G53") || prev.contains("G30") || prev == "G90");
        }
    }

    let out_unwrap = engine::run_with(&cps, &mch, stream, &json!({ "rolloverMode": "unwrapped" })).expect("unwrapped");
    assert!(out_unwrap.warnings.is_empty(), "{:?}", out_unwrap.warnings);

    let out_clamp = engine::run_with(&cps, &mch, stream, &json!({ "rotaryUnclamp": "M11", "rotaryClamp": "M10" })).expect("clamp");
    assert!(out_clamp.warnings.is_empty(), "{:?}", out_clamp.warnings);
    let cl: Vec<&str> = out_clamp.program.lines().collect();
    let i_u = cl.iter().position(|l| *l == "M11").expect("M11");
    let i_m = cl.iter().position(|l| l.starts_with("G00 ") && l.contains('A')).expect("G00 A");
    let i_c = cl.iter().position(|l| *l == "M10").expect("M10");
    assert!(i_u < i_m && i_m < i_c, "clamp order wrong:\n{}", out_clamp.program);

    let out_partial = engine::run_with(&cps, &mch, stream, &json!({ "rotaryUnclamp": "M11" })).expect("partial");
    assert!(out_partial.warnings.iter().any(|w| w.contains("rotaryClamp") || w.contains("clamp")));
}

#[test]
fn mach4_refuses_rtcp_bad_trunnion_and_simultaneous_by_name() {
    let cps = mach4_post();
    let tilted = "\
PARAM WorkOffset G54
PARAM ClearancePlane 25
TOOL 1 \"6MM EM\"
PARAM SpindleSpeed 8000
SECTION Tilted
Rapid X 0 Y 0 Z 25000 A -30000 B 0 C 0
Feed X 10000 Y 0 Z 0 F 400000 A -30000 B 0 C 0
ENDSECTION
";

    let rtcp_spec = TrunnionSpec {
        unit: "mm", rtcp: Some(true), sign_conv: "table_rh", a_zero: 0.0, c_zero: 0.0,
        c_rollover: false, pivot: [0.0, 0.0, 0.0], offset: [0.0, 0.0, 0.0],
    };
    let mch_rtcp = machine::mirror(&parse_mch(&trunnion_mch_json(&rtcp_spec)).expect("parses"));
    let out_rtcp = engine::run(&cps, &mch_rtcp, tilted).expect("runs");
    assert!(out_rtcp.warnings.iter().any(|w| w.contains("rtcp") || w.contains("RTCP")));
    assert!(!out_rtcp.program.contains("G43.4"));

    assert!(!engine::run(&cps, &default_vmc(), tilted).expect("runs").warnings.is_empty());

    let bad_cases: [(&str, Value, &str); 6] = [
        ("Single4th", json!([{ "letter": "A", "direction": [1, 0, 0], "travel": [-110, 30], "pivot": [0, 0, 0], "turnsPart": true }]), "C"),
        ("InvertedCA", json!([
            { "letter": "C", "direction": [0, 0, 1], "offset": [0, 0, 0], "turnsPart": true },
            { "letter": "A", "direction": [1, 0, 0], "parent": "C", "pivot": [0, 0, 0], "turnsPart": true }
        ]), "parent"),
        ("NoPivot", json!([
            { "letter": "A", "direction": [1, 0, 0], "turnsPart": true },
            { "letter": "C", "parent": "A", "direction": [0, 0, 1], "offset": [0, 0, 0], "turnsPart": true }
        ]), "pivot"),
        ("NoOffset", json!([
            { "letter": "A", "direction": [1, 0, 0], "pivot": [0, 0, 0], "turnsPart": true },
            { "letter": "C", "parent": "A", "direction": [0, 0, 1], "turnsPart": true }
        ]), "offset"),
        ("ParallelAxes", json!([
            { "letter": "A", "direction": [0, 0, 1], "pivot": [0, 0, 0], "turnsPart": true },
            { "letter": "C", "parent": "A", "direction": [0, 0, 1], "offset": [0, 0, 0], "turnsPart": true }
        ]), "parallel"),
        ("HeadRotary", json!([
            { "letter": "A", "direction": [1, 0, 0], "pivot": [0, 0, 0], "turnsPart": false },
            { "letter": "C", "parent": "A", "direction": [0, 0, 1], "offset": [0, 0, 0], "turnsPart": true }
        ]), "turnsPart"),
    ];
    for (name, rots, expect_substr) in bad_cases {
        let out = engine::run(&cps, &static_rotaries_mch(name, rots), tilted).expect("runs");
        assert!(
            out.warnings.iter().any(|w| w.contains(expect_substr)),
            "{name} expected `{expect_substr}` in {:?}",
            out.warnings
        );
    }

    let ok_spec = TrunnionSpec { rtcp: Some(false), ..rtcp_spec };
    let mch_ok = machine::mirror(&parse_mch(&trunnion_mch_json(&ok_spec)).expect("parses"));
    let over_travel = "\
PARAM WorkOffset G54
PARAM ClearancePlane 25
TOOL 1 \"6MM EM\"
PARAM SpindleSpeed 8000
SECTION OverTravel
Rapid X 0 Y 0 Z 25000 A -125000 B 0 C 0
Feed X 10000 Y 0 Z 0 F 400000 A -125000 B 0 C 0
ENDSECTION
";
    let out_ot = engine::run(&cps, &mch_ok, over_travel).expect("runs");
    assert!(out_ot.warnings.iter().any(|w| w.contains("travel") || w.contains("limit")));

    let simultaneous = "\
PARAM WorkOffset G54
PARAM ClearancePlane 25
TOOL 1 \"6MM EM\"
PARAM SpindleSpeed 8000
SECTION Sim5x
Rapid X 0 Y 0 Z 25000 A -30000 B 0 C 0
Feed X 10000 Y 0 Z 0 F 400000 A -30000 B 0 C 0
Feed X 20000 Y 0 Z 0 F 400000 A -45000 B 0 C 30000
ENDSECTION
";
    let out_sim = engine::run(&cps, &mch_ok, simultaneous).expect("runs");
    assert!(out_sim.warnings.iter().any(|w| w.contains("needs #1966: inverse-time feed")));
    assert!(!out_sim.program.contains("A-45") && !out_sim.program.contains("C30"));
}
