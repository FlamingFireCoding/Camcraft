//! A floor on the extraction lattice meshes to a valid solid by marching
//! tetrahedra (#4107).
//!
//! Marching tetrahedra put a crossing a few picometres from a sample whose
//! field value is almost zero (a cut floor on the lattice of cell centres).
//! The facets that share that sample kept two corners closer than the
//! 1e-9 weld, so after the weld they collapsed and their neighbours' edges
//! sat on three or more facets. `open_edges` counts only odd edges, so the
//! check here counts every edge.
//!
//! Gate map (#4107): covers sim::stock_sim::iso (tet's crossing).
//! Provenance: our own fixtures; synthetic sizes.

use std::collections::HashMap;

use camcraft_sim::cl::{Move, MoveKind};
use camcraft_sim::geometry::{Point3, Triangle};
use camcraft_sim::stock_sim::iso::{extract_with, IsoMethod};
use camcraft_sim::stock_sim::{Backend, SimOp, Stock, FIELD_BAND};

const CELL: f64 = 0.5;

/// A 30 x 30 x 20 mm block faced by four passes of a flat tool 6 mm across
/// at height `z`. The lattice samples stand at (k + 0.5) * CELL: 15.25 is
/// one, 15.1 is not.
fn faced(z: f64) -> Stock {
    let mut s = Stock::block(Backend::Field { band: FIELD_BAND }, 0.0, 0.0, 30.0, 30.0, 20.0, CELL)
        .unwrap_or_else(|e| panic!("block: {e}"));
    let moves: Vec<Move> = (0..4)
        .map(|k| {
            let y = 8.0 + 4.0 * k as f64;
            Move::new(MoveKind::Feed, Point3::new(4.0, y, z), Point3::new(26.0, y, z), Some(500.0))
        })
        .collect();
    s.simulate(&[SimOp { moves: &moves, radius: 3.0, ball: false, cone: None, corner: None }])
        .unwrap_or_else(|e| panic!("cut: {e}"));
    s
}

/// Facets that lose a corner at the 1e-9 weld, and edges that are not on
/// exactly two facets after it.
fn defects(tris: &[Triangle]) -> (usize, usize) {
    let key = |p: Point3| [(p.x * 1e9).round() as i64, (p.y * 1e9).round() as i64, (p.z * 1e9).round() as i64];
    let mut collapsed = 0;
    let mut uses: HashMap<([i64; 3], [i64; 3]), u32> = HashMap::new();
    for t in tris {
        let k = [key(t.a), key(t.b), key(t.c)];
        if k[0] == k[1] || k[1] == k[2] || k[0] == k[2] {
            collapsed += 1;
        }
        for (a, b) in [(k[0], k[1]), (k[1], k[2]), (k[2], k[0])] {
            *uses.entry(if a <= b { (a, b) } else { (b, a) }).or_insert(0) += 1;
        }
    }
    (collapsed, uses.values().filter(|&&n| n != 2).count())
}

#[test]
fn a_floor_on_the_lattice_meshes_to_a_valid_solid_by_marching_tetrahedra() {
    let s = faced(15.25);
    let f = s.field().expect("a field");
    assert!(f.distance(Point3::new(15.25, 15.25, 15.25)).abs() < 1e-9, "the floor stands on a lattice sample");
    let tris = extract_with(f, f.cell(), IsoMethod::Marching).unwrap_or_else(|e| panic!("mesh: {e}"));
    assert_eq!(defects(&tris), (0, 0), "collapsed facets, edges not on two facets");
}

#[test]
fn a_floor_off_the_lattice_stays_valid_by_both_methods() {
    let s = faced(15.1);
    let f = s.field().expect("a field");
    for m in [IsoMethod::Marching, IsoMethod::Dual] {
        let tris = extract_with(f, f.cell(), m).unwrap_or_else(|e| panic!("mesh: {e}"));
        assert_eq!(defects(&tris), (0, 0), "{m:?}");
    }
}
