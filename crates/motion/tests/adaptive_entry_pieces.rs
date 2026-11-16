//! Issue #2265: a planar pocket whose user asked for a helix entry gives
//! that entry to EVERY solid pass start - each separate piece of the
//! area, at each level, on the HSM plan and on the Offset rings it falls
//! back to - and never plunges into material without a note that names
//! the site. Fixtures: two separate square pockets, and a channel round
//! a standing part (an "outside" region). The solid reading is the
//! centre-cutting law's own (`plunge_law::column_cleared`).
//!
//! Gate map: covers camcraft_motion::machining2d (ladder_links,
//! descent_dive, pocket_hsm, pocket_emit), camcraft_motion::plunge_law.

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::load::DEFAULT_OPTIMAL_LOAD;
use camcraft_motion::machining2d::{PocketParams, PocketStrategy, pocket};
use camcraft_motion::plunge_law::column_cleared;

const D: f64 = 10.0;
const TOP: f64 = 0.0;
const LEVELS: [f64; 2] = [-4.0, -8.0];

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    let pts = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
    let Ok(l) = Loop::new(pts.iter().map(|&(x, y)| Point2::new(x, y)).collect()) else {
        panic!("fixture loop")
    };
    l
}

fn region(loops: Vec<Loop>) -> Region {
    let Ok(r) = Region::new(loops) else { panic!("fixture region") };
    r
}

fn params(strategy: PocketStrategy) -> PocketParams {
    PocketParams {
        blend_radius: None,
        tool_diameter: D,
        stepover: 4.0,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: Some(1.0),
        top_z: TOP,
        bottom_z: -8.0,
        stepdown: 4.0,
        feed: 1000.0,
        plunge_feed: 300.0,
        climb: true,
        strategy,
        entry: Entry3D::Helix,
        helix_radius: 2.0,
        optimal_load: DEFAULT_OPTIMAL_LOAD,
        blend_turns: false,
        links: camcraft_motion::links::Links::PLANAR,
    }
}

fn cut(reg: &Region, strategy: PocketStrategy) -> Toolpath {
    match pocket(reg, &params(strategy)) {
        Ok(t) => t,
        Err(e) => panic!("pocket refused: {e:?}"),
    }
}

/// Every axial plunge below the top that lands on a column no earlier cut
/// cleared at its depth: (x, y, z).
fn solid_plunges(t: &Toolpath) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for (i, m) in t.moves.iter().enumerate() {
        let axial = m.from.x == m.to.x && m.from.y == m.to.y && m.to.z < m.from.z;
        if m.kind == MoveKind::Plunge && axial && m.to.z < TOP - 1e-6 {
            let xy = Point2::new(m.to.x, m.to.y);
            if !column_cleared(&t.moves[..i], xy, m.to.z, D * 0.5) {
                out.push((m.to.x, m.to.y, m.to.z));
            }
        }
    }
    out
}

/// A descent at level `z` that is not an axial plunge (a helix turn or a
/// ramp leg) ends inside `[x0, x1]`.
fn entered_by_motion(t: &Toolpath, z: f64, x0: f64, x1: f64) -> bool {
    t.moves.iter().any(|m| {
        let axial = m.from.x == m.to.x && m.from.y == m.to.y;
        m.kind != MoveKind::Rapid && !axial && (m.to.z - z).abs() < 1e-6 && m.from.z > z + 1e-6 && (x0..=x1).contains(&m.to.x)
    })
}

/// The law: a plunge into material is never silent - a note names its
/// site (3 decimals, the ladder's own format).
fn assert_named(t: &Toolpath) {
    for (x, y, z) in solid_plunges(t) {
        let at = format!("({x:.3}, {y:.3})");
        assert!(
            t.notes.iter().any(|n| n.contains(&at) && n.contains("material")),
            "a plunge into material at {at} Z {z:.3} with no note naming it; notes: {:#?}",
            t.notes
        );
    }
}

/// Two separate pockets, HSM plan: each piece is entered by a helix at
/// each level, none by a plunge into material.
#[test]
fn each_piece_of_an_adaptive_pocket_gets_the_helix_at_each_level() {
    let reg = region(vec![rect(0.0, 0.0, 30.0, 30.0), rect(50.0, 0.0, 80.0, 30.0)]);
    let t = cut(&reg, PocketStrategy::Adaptive);
    for z in LEVELS {
        assert!(entered_by_motion(&t, z, 0.0, 30.0), "piece 1 not entered by a helix or ramp at Z {z}");
        assert!(entered_by_motion(&t, z, 50.0, 80.0), "piece 2 not entered by a helix or ramp at Z {z}");
    }
    assert!(solid_plunges(&t).is_empty(), "plunges into material: {:?}; notes: {:#?}", solid_plunges(&t), t.notes);
}

/// The Offset rings (the HSM plan's fallback) over the same two pockets:
/// the second pocket's rings are entered by the helix too.
#[test]
fn offset_rings_enter_each_piece_by_the_helix() {
    let reg = region(vec![rect(0.0, 0.0, 30.0, 30.0), rect(50.0, 0.0, 80.0, 30.0)]);
    let t = cut(&reg, PocketStrategy::OffsetRings);
    for z in LEVELS {
        assert!(entered_by_motion(&t, z, 50.0, 80.0), "piece 2 not entered by a helix or ramp at Z {z}");
    }
    assert_named(&t);
}

/// An outside region: a channel round a standing part, plus a pocket
/// apart from it. Each plunge into material is named; each piece is
/// entered by motion at each level.
#[test]
fn an_outside_channel_and_a_pocket_name_every_plunge_into_material() {
    let reg = region(vec![rect(0.0, 0.0, 40.0, 40.0), rect(14.0, 14.0, 26.0, 26.0), rect(60.0, 0.0, 85.0, 25.0)]);
    for s in [PocketStrategy::Adaptive, PocketStrategy::OffsetRings] {
        let t = cut(&reg, s);
        for z in LEVELS {
            assert!(entered_by_motion(&t, z, 0.0, 40.0), "{s:?}: the channel not entered by motion at Z {z}");
            assert!(entered_by_motion(&t, z, 60.0, 85.0), "{s:?}: the pocket not entered by motion at Z {z}");
        }
        assert_named(&t);
    }
}
