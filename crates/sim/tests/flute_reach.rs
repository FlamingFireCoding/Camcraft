//! #3525: the flute-reach law. A path whose lowest cutting tip stands
//! deeper below the stock top than the flute length is one finding, at
//! its first such move: blocking when the shank (or, past the shank, the
//! holder) is wider than the cutter, advisory when it is not. A path
//! within the flute, a rapid below it, and a tool with no stated flute
//! give none. The note names the tool, the flute and the depth.
//!
//! Gate map: covers camcraft_sim::verify (flute_reach, types).
//! Provenance: our own law; no vendor material consulted.

use camcraft_sim::cl::{Move, MoveKind, Toolpath};
use camcraft_sim::geometry::Point3;
use camcraft_sim::verify::{FixturePart, ToolGeom, Warning, flute_note, flute_reach};

fn mv(kind: MoveKind, a: (f64, f64, f64), b: (f64, f64, f64)) -> Move {
    Move {
        rotary: None,
        kind,
        from: Point3::new(a.0, a.1, a.2),
        to: Point3::new(b.0, b.1, b.2),
        feed: if kind == MoveKind::Rapid { None } else { Some(800.0) },
        arc: None,
        axis: None,
    }
}

/// Two levels of the #3319 heights under a stock top at Z 22: Z 12 and
/// Z 1.5, each a rapid drop, a plunge and one cut.
fn levels() -> Toolpath {
    let mut moves = Vec::new();
    for z in [12.0, 1.5] {
        moves.push(mv(MoveKind::Rapid, (0.0, 0.0, 32.0), (0.0, 0.0, 24.0)));
        moves.push(mv(MoveKind::Feed, (0.0, 0.0, 24.0), (0.0, 0.0, z)));
        moves.push(mv(MoveKind::Feed, (0.0, 0.0, z), (30.0, 0.0, z)));
        moves.push(mv(MoveKind::Rapid, (30.0, 0.0, z), (30.0, 0.0, 32.0)));
    }
    Toolpath { moves, cycles: Vec::new(), notes: Vec::new(), comps: Vec::new(), threads: Vec::new(), probes: Vec::new(), breaks: Vec::new(), plasma: None }
}

/// An 8 mm cutter with `flute` mm of flute and a `shank_d` shank 40 mm
/// long under a 44 mm holder.
fn mill(flute: f64, shank_d: f64) -> ToolGeom {
    ToolGeom { radius: 4.0, nose_radius: 0.0, flute_len: flute, shank_radius: shank_d / 2.0, shank_len: 40.0, holder_radius: 22.0, holder_len: 35.0 }
}

#[test]
fn a_wider_shank_past_the_flute_blocks_at_the_first_deep_move() {
    let w = flute_reach(&levels(), &mill(10.0, 10.0), 22.0);
    let Some(Warning::FluteShort { at, flute_mm, depth_mm, wider }) = w else {
        panic!("a finding: {w:?}");
    };
    // The plunge to Z 12 is 10 mm deep: within the flute. The plunge to
    // Z 1.5 (move 5) is the first move past it.
    assert_eq!(at, 5);
    assert_eq!(flute_mm, 10.0);
    assert!((depth_mm - 20.5).abs() < 1e-9, "{depth_mm}");
    assert_eq!(wider, Some(FixturePart::Shank));
    let Some(w) = w else { return };
    assert!(w.blocks_post());
    let text = w.to_string();
    assert!(text.contains("flute length is 10.000 mm") && text.contains("20.500 mm below the stock top from move 5"), "{text}");
    assert!(text.contains("the shank, wider than the cutter, is in the cut"), "{text}");
    let note = flute_note("EM-0800-4FL", &w).unwrap_or_default();
    assert!(note.starts_with("EM-0800-4FL has a flute length of 10.000 mm; this operation cuts 20.500 mm below the stock top; the shank"), "{note}");
}

#[test]
fn a_flute_longer_than_the_cut_gives_no_finding() {
    assert_eq!(flute_reach(&levels(), &mill(25.0, 10.0), 22.0), None);
    // Exactly the depth: within the flute.
    assert_eq!(flute_reach(&levels(), &mill(20.5, 10.0), 22.0), None);
}

#[test]
fn a_same_diameter_shank_past_the_flute_is_advisory() {
    let w = flute_reach(&levels(), &mill(10.0, 8.0), 22.0);
    assert!(matches!(w, Some(Warning::FluteShort { wider: None, .. })), "{w:?}");
    let Some(w) = w else { return };
    assert!(!w.blocks_post());
    assert!(w.to_string().contains("the post is not held"), "{w}");
    assert!(flute_note("T1", &w).is_some_and(|n| n.contains("can rub the walls")));
}

#[test]
fn a_reduced_shank_with_a_wider_holder_past_it_blocks_as_the_holder() {
    // Flute 5, shank 10 long: the 20.5 mm cut reaches the 44 mm holder.
    let tool = ToolGeom { shank_len: 10.0, ..mill(5.0, 7.0) };
    let w = flute_reach(&levels(), &tool, 22.0);
    assert!(matches!(w, Some(Warning::FluteShort { wider: Some(FixturePart::Holder), .. })), "{w:?}");
    assert!(w.is_some_and(|w| w.blocks_post()));
}

#[test]
fn rapids_and_unstated_flutes_are_not_read() {
    // Only rapids reach under the flute: no cutting move, no finding.
    let mut p = levels();
    p.moves.retain(|m| m.kind == MoveKind::Rapid);
    assert_eq!(flute_reach(&p, &mill(10.0, 10.0), 22.0), None);
    // No stated flute, or a top that is not finite.
    assert_eq!(flute_reach(&levels(), &mill(0.0, 10.0), 22.0), None);
    assert_eq!(flute_reach(&levels(), &mill(f64::NAN, 10.0), 22.0), None);
    assert_eq!(flute_reach(&levels(), &mill(10.0, 10.0), f64::INFINITY), None);
    // The note speaks only for its own kind.
    assert_eq!(flute_note("T1", &Warning::RapidInStock { at: 0 }), None);
}
