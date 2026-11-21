//! Sharp inside corners at the wire radius (#3207): a sharp square die
//! is refused by default, and cut when the operation states that its
//! inside corners may keep the wire's own radius - the leftover said
//! as a note. A rest past the wire's radius stays refused.
//!
//! The wire and gap are SYNTHETIC: the 0.25 mm wire of the report and
//! a 0.037 mm gap, so the kerf offset is the report's 0.162 mm.

use super::*;

const K: f64 = 0.162;

fn params(side: WireSide, start: (f64, f64), sharp: bool) -> WireParams {
    WireParams {
        wire_diameter: 0.25,
        gap: 0.037,
        condition: "C1".into(),
        side,
        start: Some(Point2::new(start.0, start.1)),
        plane_z: 0.0,
        tabs: None,
        stock: None,
        clearance: 1.0,
        sharp_corners: sharp,
    }
}

fn square(h: f64) -> Loop {
    Loop::new(vec![Point2::new(-h, -h), Point2::new(h, -h), Point2::new(h, h), Point2::new(-h, h)]).unwrap()
}

/// The report's part: a sharp 10 mm square die opening.
#[test]
fn a_sharp_square_die_is_cut_when_sharp_inside_corners_are_stated() {
    let die = square(5.0);
    // The default stays as it was: refused by name.
    let why = wire_contour(std::slice::from_ref(&die), &[], &params(WireSide::Die, (0.0, 0.0), false)).expect_err("refused").to_string();
    assert!(why.contains("inside corner") && why.contains("0.1620") && why.contains("0.0671"), "{why}");
    // Stated: cut, and the leftover in each corner said by its size.
    let path = wire_contour(&[die], &[], &params(WireSide::Die, (0.0, 0.0), true)).expect("a sharp die is cut");
    let rest = (2.0_f64.sqrt() - 1.0) * K;
    let note = path.notes.iter().find(|n| n.contains("sharp inside corners")).unwrap_or_else(|| panic!("{:?}", path.notes));
    assert!(note.contains(&format!("{K:.4}")) && note.contains(&format!("{rest:.4}")), "{note}");
    // The wire still stands the kerf offset off every side: the corner
    // of its path is the square's corner moved in by K on both axes.
    let corner = path.moves.iter().any(|m| (m.to.x - (5.0 - K)).abs() < 1e-9 && (m.to.y - (5.0 - K)).abs() < 1e-9);
    assert!(corner, "the wire's corner at X{:.3} Y{:.3}", 5.0 - K, 5.0 - K);
    // A punch with the option set has no inside corner: no note.
    let punch = wire_contour(&[square(5.0)], &[], &params(WireSide::Punch, (-10.0, 0.0), true)).expect("a punch");
    assert!(!punch.notes.iter().any(|n| n.contains("sharp inside corners")), "{:?}", punch.notes);
}

/// A corner so acute that the rest stands past the wire's own radius
/// is still refused with the option set: a 30-degree inside corner
/// leaves K / sin(15 deg) - K, about 2.9 K.
#[test]
fn an_inside_corner_past_the_wire_radius_stays_refused() {
    let tri = Loop::new(vec![Point2::new(0.0, 0.0), Point2::new(20.0, -5.358_983_848), Point2::new(20.0, 5.358_983_848)]).unwrap();
    let why = wire_contour(&[tri], &[], &params(WireSide::Die, (15.0, 0.0), true)).expect_err("refused").to_string();
    assert!(why.contains("sharp inside corners") && why.contains("wire's own corner radius"), "{why}");
}
