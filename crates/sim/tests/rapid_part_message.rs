//! Provenance: original box fixtures for the shop rule that rapids must clear the part and stock.
//! Gate map (#3520): covers camcraft_sim::verify::{solid,rapids} and Warning messages.

use camcraft_sim::cl::{Cycle, CycleKind, Move, MoveKind, Toolpath};
use camcraft_sim::geo2d::{Loop, Point2, Region};
use camcraft_sim::geometry::{Point3, cube};
use camcraft_sim::verify::{Cutter, Mode, SolidLaw, ToolGeom, VerifyContext, Warning, solid_hits, stack_hits, verify};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn cutter() -> Cutter {
    Cutter {
        radius: 3.0, corner_r: 0.0, tip_angle: 0.0, flute_len: 20.0,
        shank_radius: 2.0, shank_len: 30.0, holder_radius: 15.0, holder_len: 40.0,
        body: None, neck: None, undercut: None,
    }
}

fn law() -> SolidLaw {
    SolidLaw { step: 0.25, allow: 0.01, hole_walls: false }
}

fn assert_part_message(warnings: &[Warning], at: usize) {
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(warnings[0].to_string(), format!("tool meets the part on rapid move {at}"));
    assert!(warnings[0].blocks_post(), "a rapid into the part must hold the post");
    assert_eq!(serde_json::to_value(warnings[0]).unwrap(), serde_json::json!({"rapid_in_part": {"at": at}}));
}

#[test]
fn a_vertical_part_rapid_names_the_part_and_move() {
    let part = cube(p(0.0, 0.0, 0.0), p(40.0, 40.0, 10.0));
    let path = Toolpath::new(vec![Move::new(MoveKind::Rapid, p(15.0, 30.0, 5.0), p(15.0, 30.0, 32.0), None)]);
    assert_part_message(&solid_hits(&path, &part.facets, &cutter(), &law()), 0);
}

#[test]
fn a_stack_retract_edge_column_names_the_part_and_move() {
    // The cutting edge starts below the slab, then rises through it.
    let slab = cube(p(5.0, 22.0, 40.0), p(25.0, 38.0, 45.0));
    let bare = Cutter { shank_len: 0.0, holder_len: 0.0, ..cutter() };
    let path = Toolpath::new(vec![
        Move::new(MoveKind::Feed, p(15.0, 10.0, 9.7), p(15.0, 30.0, 9.7), Some(500.0)),
        Move::new(MoveKind::Rapid, p(15.0, 30.0, 9.7), p(15.0, 30.0, 60.0), None),
    ]);
    assert_part_message(&stack_hits(&path, &slab.facets, &bare, &law()), 1);
}

#[test]
fn a_stopped_fine_bore_holder_rapid_names_the_part_and_move() {
    // The holder starts below the slab. Its stopped retract enters it.
    let slab = cube(p(-20.0, -20.0, 100.0), p(20.0, 20.0, 105.0));
    let path = Toolpath {
        cycles: vec![Cycle {
            kind: CycleKind::FineBore, xy: Point2::new(0.0, 0.0), r_z: 20.0,
            bottom_z: 0.0, entry_z: 20.0, peck: 0.0, dwell: 0.0, first_move: 0,
        }],
        ..Toolpath::new(vec![
            Move::new(MoveKind::Plunge, p(0.0, 0.0, 20.0), p(0.0, 0.0, 0.0), Some(60.0)),
            Move::new(MoveKind::Rapid, p(0.0, 0.0, 0.0), p(0.5, 0.0, 0.0), None),
            Move::new(MoveKind::Rapid, p(0.5, 0.0, 0.0), p(0.5, 0.0, 20.0), None),
            Move::new(MoveKind::Rapid, p(0.5, 0.0, 20.0), p(0.0, 0.0, 20.0), None),
        ])
    };
    let warnings = solid_hits(&path, &slab.facets, &cutter(), &law());
    // The feed and shift back also meet the slab; isolate the stopped lift.
    let lift: Vec<_> = warnings.into_iter().filter(|w| {
        let value = serde_json::to_value(w).unwrap();
        value.as_object().unwrap().values().any(|fields| fields["at"] == 2)
    }).collect();
    assert_part_message(&lift, 2);
}

#[test]
fn a_stock_rapid_keeps_its_stock_message() {
    let region = Region::new(vec![Loop::new(vec![
        Point2::new(-40.0, -40.0), Point2::new(40.0, -40.0),
        Point2::new(40.0, 40.0), Point2::new(-40.0, 40.0),
    ]).unwrap()]).unwrap();
    let tool = ToolGeom {
        radius: 2.0, nose_radius: 0.0, flute_len: 30.0, shank_radius: 2.0,
        shank_len: 20.0, holder_radius: 2.0, holder_len: 1.0,
    };
    let path = Toolpath::new(vec![Move::new(MoveKind::Rapid, p(0.0, 0.0, 35.0), p(0.0, 0.0, 20.0), None)]);
    let cx = VerifyContext {
        region: &region, stock: Some(&region), tool: &tool, mode: Mode::Area,
        stock_top_z: 25.0, clearance_z: 35.0, gouge_tol: 0.01, z_tol: 1e-3,
        ..Default::default()
    };
    let warnings = verify(&path, &cx);
    assert_eq!(warnings, vec![Warning::RapidInStock { at: 0 }]);
    assert_eq!(warnings[0].to_string(), "rapid move 0 travels through stock (below the clearance plane, or at it below the stock top)");
    assert!(warnings[0].blocks_post());
    assert_eq!(serde_json::to_value(warnings[0]).unwrap(), serde_json::json!({"rapid_in_stock": {"at": 0}}));
}
