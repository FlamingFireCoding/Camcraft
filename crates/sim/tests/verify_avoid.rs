//! Unit tests for `sim::verify::avoid` (#2055): 3D clearance across
//! flutes, shank, holder, axial faces, and 3D edges.
//! Provenance: our own geometry checks; no vendor source consulted.

use camcraft_sim::cl::{MoveKind, PathBuilder, Toolpath};
use camcraft_sim::geo2d::Point2;
use camcraft_sim::geometry::{Point3, Triangle};
use camcraft_sim::tools::ToolGeom;
use camcraft_sim::verify::{
    AVOID_PAD, AVOID_VERIFY_TOL, AvoidItem, AvoidSpec, Cutter, FixturePart, Warning, avoid_hits,
};

fn stepped_cutter() -> Cutter {
    Cutter::flat(&ToolGeom {
        radius: 3.0,
        nose_radius: 0.0,
        flute_len: 10.0,
        shank_radius: 6.0,
        shank_len: 12.0,
        holder_radius: 15.0,
        holder_len: 20.0,
    })
}

fn seg_path(a: [f64; 3], b: [f64; 3]) -> Toolpath {
    let mut pb = PathBuilder::new(Point3::new(a[0], a[1], a[2]));
    pb.push(MoveKind::Feed, Point3::new(b[0], b[1], b[2]), Some(500.0));
    pb.finish()
}

#[test]
fn detects_axial_holder_and_edge_clearance_violations() {
    let cutter = stepped_cutter();
    // Vertical wall at x = 0, y in [-5, 5], z in [0, 30], plus horizontal edge at y = -15, z = 10 over [-20..-10].
    let item = AvoidItem {
        label: "tower wall".into(),
        tris: vec![
            Triangle::new(Point3::new(0.0, -5.0, 0.0), Point3::new(0.0, 5.0, 0.0), Point3::new(0.0, 5.0, 30.0)),
            Triangle::new(Point3::new(0.0, -5.0, 0.0), Point3::new(0.0, 5.0, 30.0), Point3::new(0.0, -5.0, 30.0)),
        ],
        edges: vec![([-20.0, -15.0, 10.0], [-10.0, -15.0, 10.0])],
    };
    let spec = AvoidSpec { items: vec![item], cutter, clearance: 1.5 };

    // Move at z = 5, x = -15: flute r=3 is 12 mm from wall (ok), shank r=6 is 9 mm (ok),
    // HOLDER (z = 27..47, r = 15) hits wall z = 30 within 1.5 mm clearance (short = 1.5)!
    let hit_holder = avoid_hits(&seg_path([-15.0, -2.0, 5.0], [-15.0, 2.0, 5.0]), &spec, AVOID_VERIFY_TOL)
        .expect("verify runs");
    assert_eq!(hit_holder.len(), 1);
    assert!(matches!(hit_holder[0], Warning::AvoidHit { part: FixturePart::Holder, .. }), "{hit_holder:?}");

    // Move at z = 11.0 passing directly over edge at y = -15, z = 10.0 (only 1.0 mm axial clearance < 1.5):
    let hit_axial = avoid_hits(&seg_path([-15.0, -15.0, 11.0], [-14.0, -15.0, 11.0]), &spec, AVOID_VERIFY_TOL)
        .expect("verify runs");
    assert_eq!(hit_axial.len(), 1, "axial edge violation must be caught: {hit_axial:?}");

    // Move kept out at the generator's own keep-out distance passes clean:
    let ko = spec.keepout_region(5.0, 5.0, AVOID_PAD).expect("keepout");
    assert!(ko.contains(Point2::new(-16.0, 0.0)), "holder + clearance covers x = -16");
    let clear = avoid_hits(&seg_path([-16.6, -2.0, 5.0], [-16.6, 2.0, 5.0]), &spec, AVOID_VERIFY_TOL)
        .expect("verify runs");
    assert!(clear.is_empty(), "clear of holder + clearance: {clear:?}");
}

#[test]
fn detects_zero_clearance_graze_and_link_obstruction() {
    let cutter = stepped_cutter();
    let item = AvoidItem {
        label: "pin".into(),
        tris: vec![
            Triangle::new(Point3::new(0.0, 0.0, 0.0), Point3::new(4.0, 0.0, 0.0), Point3::new(4.0, 4.0, 0.0)),
            Triangle::new(Point3::new(0.0, 0.0, 0.0), Point3::new(4.0, 4.0, 0.0), Point3::new(0.0, 4.0, 0.0)),
        ],
        edges: vec![([0.0, 25.0, 25.0], [4.0, 25.0, 25.0])],
    };
    let spec = AvoidSpec { items: vec![item], cutter, clearance: 0.0 };

    // Flute r = 3 touching edge/face boundary at x = -3.0 (distance = 0.0):
    let touch = avoid_hits(&seg_path([-3.0, 1.0, 0.0], [-3.0, 3.0, 0.0]), &spec, AVOID_VERIFY_TOL)
        .expect("verify runs");
    assert_eq!(touch.len(), 1, "zero-distance touch must block: {touch:?}");

    // Outside by 0.02 mm passes clean:
    let ok = avoid_hits(&seg_path([-3.02, 1.0, 0.0], [-3.02, 3.0, 0.0]), &spec, AVOID_VERIFY_TOL)
        .expect("verify runs");
    assert!(ok.is_empty(), "{ok:?}");

    // Rapid link crossing over edge at y = 25, z = 25 requires z_req > 25:
    let (h, idx) = spec.link_obstruction(Point2::new(-5.0, 25.0), Point2::new(8.0, 25.0), AVOID_PAD)
        .expect("link crosses pin");
    assert_eq!(idx, 1);
    assert!(h > 25.0, "must clear top z = 25: {h}");
}
