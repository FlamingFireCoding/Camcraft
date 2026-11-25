//! The multi-band and single-chain laws (#2829, the third #2804
//! slice): three ordered guide chains machine two bands whose shared
//! boundary chain is cut once; a single helical chain covers a full
//! revolution as its own next lap; and a ring that crosses a corner
//! chain is refused by name, at the crossing's own spot.
//!
//! Gate map (M4.4): covers camcraft_motion::multiaxis (rotary_flow
//! and rotary_flow_set, the #2829 laws) on this crate's own
//! fixtures.

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::geometry::Point3;
use camcraft_motion::multiaxis::{rotary_flow, AxisControl};

use crate::fixture::*;

/// The groove's three root-line helices at v-offsets 0, half and the
/// full floor width - all exact stations of the floor strip's own
/// vertex columns, so every point is on the model.
fn root_trio(n_circ: usize, turns: usize, phase: f64) -> Vec<Vec<Point3>> {
    let step = TAU / n_circ as f64;
    let mut out = Vec::with_capacity(3);
    for f in [0.0, 0.5, 1.0] {
        let mut c = Vec::with_capacity(n_circ * turns + 1);
        for j in 0..=n_circ * turns {
            let th = phase + step * j as f64;
            c.push(gp(R_ROOT, th, law(th) + W_V * f));
        }
        out.push(c);
    }
    out
}

/// The landing station of every pass: the ball centre of each plunge.
fn landings(path: &Toolpath, r: f64) -> Vec<Point3> {
    path.moves
        .iter()
        .filter(|m| m.kind == MoveKind::Plunge)
        .filter_map(|m| center_of(m, r))
        .collect()
}

#[test]
fn three_ordered_chains_cut_two_bands_and_the_boundary_once() {
    // One turn of the groove fixture, three root-line helices picked
    // in band order. Each 2.0 mm band takes ceil(2.0 / 0.9) + 1 = 4
    // passes; the shared middle chain is band 1's own last pass
    // already, so band 2 starts at its first interior pass: 4 + 4 -
    // 1 = 7 passes, and no two of them land on one station.
    let facets = groove(N_CIRC, 1, 0.0);
    let chains = root_trio(N_CIRC, 1, 0.0);
    let tool = ball(2.0);
    let path = rotary_flow(
        &chains,
        &facets,
        &stated_axis(),
        &params(&tool, Some(0.9)),
        &AxisControl::default(),
    )
    .unwrap();
    no_gouge(&path, &facets, 1.0);
    no_swept_gouge(&path, &facets, 1.0);
    let ls = landings(&path, 1.0);
    assert_eq!(
        ls.len(),
        7,
        "the two bands did not divide into 7 passes: {:?}",
        path.notes
    );
    for (i, &x) in ls.iter().enumerate() {
        for &y in ls.iter().skip(i + 1) {
            let d = x - y;
            assert!(
                d.dot(d) > 1e-12,
                "two passes land on one station - the boundary is cut twice: {x:?} ~ {y:?}"
            );
        }
    }
    assert!(
        path.notes.iter().any(|n| n.contains("boundary chain shared with band 1")),
        "the notes do not state the shared boundary: {:?}",
        path.notes
    );
}

#[test]
fn one_helical_chain_covers_the_full_revolution() {
    // The single-chain case (#2829): the near root edge alone, one
    // full turn. The kernel morphs it to its own next lap, so the
    // developed band is exactly one revolution at the chain's own
    // radius: ceil(R_ROOT * TAU / 2.0) = 26 spans, one pass each -
    // the picked chain at the opening edge, the closing pass dropped
    // because the lap's two edges are one world curve. The first
    // pass is the picked chain itself: one full lap wound.
    let facets = groove(N_CIRC, 1, 0.0);
    let mut pair = root_pair(N_CIRC, 1, 0.0);
    let alone = pair.swap_remove(0);
    let tool = ball(6.0);
    let mut q = params(&tool, Some(2.0));
    q.clearance = 20.0; // the wall-adjacent pass pushes past the 14 mm helper's read
    let path = rotary_flow(&[alone], &facets, &stated_axis(), &q, &AxisControl::default())
        .unwrap();
    no_gouge(&path, &facets, 3.0);
    let ls = landings(&path, 3.0);
    assert_eq!(
        ls.len(),
        26,
        "the full revolution did not divide into 26 passes: {:?}",
        path.notes
    );
    let centers = first_pass_centers(&path, 20.0, 3.0);
    let span = wound(&centers);
    assert!(
        (span - TAU).abs() < 0.05,
        "the first pass winds {span:.3} rad, not one full revolution"
    );
    assert!(
        path.notes.iter().any(|n| n.contains("morphed a full revolution")),
        "the notes do not state the lap morph: {:?}",
        path.notes
    );
}

#[test]
fn a_ring_that_crosses_a_corner_chain_is_refused_by_name() {
    // A closed hoop on the prism crosses the corner chain at its own
    // middle: the set law refuses the pair at the crossing, never
    // blends a band that folds through it.
    let facets = prism(12, R_ROOT, 0.0, 4.0);
    let corner: Vec<Point3> = (0..=4).map(|k| gp(R_ROOT, 0.0, k as f64)).collect();
    let hoop = ring(12, R_ROOT, 2.0);
    let tool = ball(2.0);
    let e = rotary_flow(
        &[corner, hoop],
        &facets,
        &stated_axis(),
        &params(&tool, Some(0.9)),
        &AxisControl::default(),
    )
    .unwrap_err();
    let msg = format!("{e}");
    assert!(msg.contains("meet or cross each other"), "{msg}");
    let at = e.location().expect("the refusal names its spot");
    assert!(
        (at.x - R_ROOT).abs() < 1e-6 && at.y.abs() < 1e-6 && (at.z - 2.0).abs() < 1e-6,
        "the refusal stands at {at:?}, not at the crossing"
    );
}
