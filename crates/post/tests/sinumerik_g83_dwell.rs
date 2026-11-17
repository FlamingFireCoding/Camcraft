//! Sinumerik G83 dwell refusal, from Siemens' published SINUMERIK 840D sl ISO Milling manual (#2349).
//! Gate map: covers assets/posts/sinumerik.cps, camcraft_post::post::engine
//! and camcraft_post::post::cnc.

use camcraft_post::cl::{CycleKind, MoveKind};
use camcraft_post::post::{
    cnc::{self, CycleRec, MoveRec, Record},
    engine,
};
use serde_json::json;
use std::path::Path;

fn read(rel: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel))
        .expect("fixture")
}

fn stream(kind: CycleKind, dwell: f64) -> String {
    let records = vec![
        Record::Param { name: "SpindleSpeed".into(), value: json!(900.0) },
        Record::Param { name: "ClearancePlane".into(), value: json!(10.0) },
        Record::Param { name: "RetractPlane".into(), value: json!(10.0) },
        Record::Tool { number: 4, name: "DRILL".into() },
        Record::Section { begin: true },
        Record::Move(MoveRec {
            kind: MoveKind::Rapid,
            x: 10.0,
            y: 10.0,
            z: 2.0,
            feed: None,
            center: None,
            ccw: false,
            plane: Default::default(),
            abc: None,
        }),
        Record::Cycle(CycleRec {
            kind,
            x: 10.0,
            y: 10.0,
            r_z: 2.0,
            bottom_z: -12.0,
            entry_z: 0.0,
            peck: 3.0,
            dwell,
            first_move: 1,
            feed: Some(120.0),
        }),
        Record::Section { begin: false },
    ];
    cnc::serialize(&records)
}

fn run(kind: CycleKind, dwell: f64) -> engine::PostOutput {
    let machine = serde_json::from_str(&read("assets/machines/3axis-vmc.mch")).expect("machine");
    engine::run(&read("assets/posts/sinumerik.cps"), &machine, &stream(kind, dwell))
        .expect("post engine")
}

fn lines_with<'a>(program: &'a str, word: &str) -> Vec<&'a str> {
    program
        .lines()
        .filter(|line| line.split_whitespace().any(|w| w == word))
        .collect()
}

#[test]
fn sinumerik_refuses_g83_dwell() {
    // The stream carries whole ms: 0.001 s is the smallest dwell it holds.
    for dwell in [0.001, 0.5] {
        let out = run(CycleKind::Deep, dwell);
        assert!(
            out.warnings.iter().any(|w| w.contains("G83") && w.contains("dwell")),
            "G83 dwell {dwell} must be refused by name: {:?}",
            out.warnings
        );
        assert!(
            lines_with(&out.program, "G83").is_empty(),
            "a refused G83 must not be written:\n{}",
            out.program
        );
    }
}

#[test]
fn sinumerik_g83_without_dwell_is_unchanged() {
    let out = run(CycleKind::Deep, 0.0);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let lines = lines_with(&out.program, "G83");
    assert_eq!(lines.len(), 1, "{}", out.program);
    assert!(
        !lines[0].split_whitespace().any(|w| w.starts_with('P')),
        "G83 must carry no P word: {}",
        lines[0]
    );
    assert!(lines[0].split_whitespace().any(|w| w == "Q3"), "{}", lines[0]);
}

#[test]
fn sinumerik_g82_dwell_still_posts() {
    // The refusal is G83 only: a spot-drill dwell keeps its G82 P word.
    let out = run(CycleKind::Drill, 0.5);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    let lines = lines_with(&out.program, "G82");
    assert_eq!(lines.len(), 1, "{}", out.program);
    assert!(lines[0].split_whitespace().any(|w| w == "P500"), "{}", lines[0]);
}
