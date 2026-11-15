//! Provenance: synthetic streams from our CNC intermediate contract (#1496).
//! Gate map: covers assets/posts/sodick-wire.cps, camcraft_post::post::engine,
//! camcraft_post::post::cnc.

use camcraft_post::cl::{ArcPlane, MoveKind};
use camcraft_post::post::cnc::{self, MoveRec, Record};
use camcraft_post::post::engine;
use serde_json::json;

const SCRIPT: &str = include_str!("../../../assets/posts/sodick-wire.cps");

fn mv(kind: MoveKind, x: f64) -> Record {
    Record::Move(MoveRec {
        kind,
        x,
        y: 0.0,
        z: 0.0,
        feed: (kind == MoveKind::Feed).then_some(2.0),
        center: None,
        ccw: false,
        plane: ArcPlane::Xy,
        abc: None,
    })
}

fn run(angle: Option<f64>) -> engine::PostOutput {
    let mut records = vec![
        Record::Param {
            name: "OperationName".into(),
            value: json!("Punch1"),
        },
        Record::Param {
            name: "WorkOffset".into(),
            value: json!("G54"),
        },
        Record::Param {
            name: "EdmCondition".into(),
            value: json!("C110"),
        },
        Record::Section { begin: true },
        mv(MoveKind::Rapid, -30.0),
    ];
    if let Some(angle) = angle {
        // A direct post stream must not silently flatten this move when
        // the section's WireTaper parameter is absent.
        records.push(Record::Param {
            name: "WireAngle".into(),
            value: json!(angle),
        });
    }
    records.extend([mv(MoveKind::Feed, -20.0), Record::Section { begin: false }]);
    let machine = json!({"kind": "wire_edm", "name": "Synthetic wire", "edm": {"awt": true}});
    engine::run(SCRIPT, &machine, &cnc::serialize(&records)).expect("synthetic stream runs")
}

#[test]
fn sodick_refuses_angle_words_even_without_section_taper() {
    for angle in [3.0, -3.0, 0.0] {
        let out = run(Some(angle));
        assert!(
            out.warnings
                .iter()
                .any(|w| w.contains("Punch1") && w.contains("angle-word")),
            "angle {angle}: {:?}\n{}",
            out.warnings,
            out.program
        );
        for word in ["G01", "T91", "C110"] {
            assert!(
                !out.program
                    .lines()
                    .filter(|line| !line.starts_with('('))
                    .any(|line| line.split_whitespace().any(|w| w == word)),
                "angle {angle}: {}",
                out.program
            );
        }
    }
}

#[test]
fn sodick_vertical_stream_still_posts_without_angle_words() {
    let out = run(None);
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    assert!(
        out.program.lines().any(|line| line == "G01 X-20. Y0."),
        "{}",
        out.program
    );
}
