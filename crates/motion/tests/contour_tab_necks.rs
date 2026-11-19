//! 2D Contour tabs against every cut of their level (#2561): the neck of each
//! tab, read from the material that the posted moves leave (shape mask), for
//! a tab beside a U-turn of its own ring and a tab beside a second ring.
//!
//! Gate map (M4.4): covers camcraft_motion::machining2d (contour, tab_bridge, tab_clear).

mod common;

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{ContourParams, Lead, Side, contour};
use common::tab_neck::{Neck, tab_necks};

const D: f64 = 6.0;
const W: f64 = 4.0;
const BOTTOM: f64 = -4.0;
const TAB_H: f64 = 1.0;
const CELL: f64 = 0.1;

fn lp(pts: &[(f64, f64)]) -> Loop {
    Loop::new(pts.iter().map(|&(x, y)| Point2::new(x, y)).collect()).unwrap()
}

fn params(count: u32) -> ContourParams {
    ContourParams {
        tool_diameter: D,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: 2.0,
        top_z: 0.0,
        bottom_z: BOTTOM,
        stepdown: 2.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        side: Side::Outside,
        lead: Lead::None,
        tolerance: 0.01,
        tab_count: count,
        tab_width: W,
        tab_height: TAB_H,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        links: Links::PLANAR,
    }
}

fn necks(part: &Region, t: &Toolpath) -> Vec<Neck> {
    tab_necks(part, t, D, BOTTOM + TAB_H, CELL).unwrap()
}

/// A 60 x 40 part with a notch `g` wide and 25 deep in its top edge.
fn notched(g: f64) -> Region {
    let (a, b) = (30.0 - g * 0.5, 30.0 + g * 0.5);
    Region::new(vec![lp(&[(0.0, 0.0), (60.0, 0.0), (60.0, 40.0), (b, 40.0), (b, 15.0), (a, 15.0), (a, 40.0), (0.0, 40.0)])]).unwrap()
}

/// Two parts `g` apart: 30 x 30, and 30 x 50 beside it.
fn pair(g: f64) -> Region {
    Region::new(vec![
        lp(&[(0.0, 0.0), (30.0, 0.0), (30.0, 30.0), (0.0, 30.0)]),
        lp(&[(30.0 + g, 0.0), (60.0 + g, 0.0), (60.0 + g, 50.0), (30.0 + g, 50.0)]),
    ])
    .unwrap()
}

/// The meter reads the tab width to about one cell.
const TOL: f64 = 0.25;

/// A loop is held: the note names the cut that reaches its tab.
fn held(t: &Toolpath) -> usize {
    t.notes.iter().filter(|s| s.starts_with("tabs:") && s.contains("cuts into it") && s.contains("cut nothing on this loop")).count()
}

/// The number of tabs, each with a neck of the tab width or more.
fn all_hold(part: &Region, t: &Toolpath, what: &str) -> usize {
    let n = necks(part, t);
    for k in &n {
        assert!(k.width >= W - TOL, "{what}: the tab at ({:.2}, {:.2}) has a neck of {:.3} mm, under the tab width {W}: {:?}", k.at.x, k.at.y, k.width, t.notes);
    }
    n.len()
}

/// Nothing is cut under the tab top.
fn cuts_nothing_at_the_tabs(t: &Toolpath) -> bool {
    t.moves.iter().all(|m| m.kind == MoveKind::Rapid || m.from.z.min(m.to.z) >= BOTTOM + TAB_H - 1e-9)
}

#[test]
fn the_meter_reads_the_tab_width_on_a_plain_part() {
    let part = Region::new(vec![lp(&[(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)])]).unwrap();
    let t = contour(&part, &params(4)).unwrap();
    assert_eq!(all_hold(&part, &t, "plain"), 4);
    assert!(necks(&part, &t).iter().all(|k| k.width <= W + TOL), "the tabs are not wider than asked");
    assert!(t.notes.iter().all(|s| !s.starts_with("tabs:")), "{:?}", t.notes);
}

/// The notched part with two tabs, the second one on the notch wall.
fn notch_path(g: f64) -> (Region, Toolpath) {
    let part = notched(g);
    let mut p = params(2);
    p.links.approach.start_point = Some(Point2::new(30.0 - g * 0.5 - D * 0.5, 30.0));
    let t = contour(&part, &p).unwrap();
    (part, t)
}

#[test]
fn a_tab_that_a_u_turn_of_its_ring_cuts_into_holds_the_loop() {
    // A 10 mm notch and a 6 mm cutter: the ring goes down the notch and
    // comes back 4 mm beside itself. The return run cut the tab on the
    // notch wall off its stock (a neck of 0), with no note.
    let (part, t) = notch_path(10.0);
    assert_eq!(held(&t), 1, "{:?} necks {:?}", t.notes, necks(&part, &t));
    assert!(t.notes.iter().any(|s| s.contains("passes 4.000 mm from the tab")), "{:?}", t.notes);
    assert!(cuts_nothing_at_the_tabs(&t), "a held loop cuts nothing under the tab top");
}

#[test]
fn a_tab_beside_a_narrow_strip_is_named() {
    // A 14 mm notch: the two runs are 8 mm apart, so 2 mm of stock stands
    // between the kerfs. The tab keeps its width; the strip beside it is
    // narrower, and a note says so.
    let (part, t) = notch_path(14.0);
    assert_eq!(held(&t), 0, "{:?}", t.notes);
    assert!(t.notes.iter().any(|s| s.starts_with("tabs:") && s.contains("leaves only 2.000 mm of material beside it")), "{:?}", t.notes);
    let n = necks(&part, &t);
    assert_eq!(n.len(), 2, "{n:?}");
    let narrow = n.iter().map(|k| k.width).fold(f64::INFINITY, f64::min);
    assert!((narrow - 2.0).abs() <= TOL, "the strip is the neck: {n:?}");
}

#[test]
fn a_tab_with_room_beside_it_has_no_note() {
    // A 20 mm notch: 8 mm of stock between the kerfs, the tab width twice.
    let (part, t) = notch_path(20.0);
    assert_eq!(all_hold(&part, &t, "wide notch"), 2);
    assert!(t.notes.iter().all(|s| !s.starts_with("tabs:")), "{:?}", t.notes);
    let both = pair(20.0);
    let t = contour(&both, &params(4)).unwrap();
    assert_eq!(all_hold(&both, &t, "wide pair"), 8);
    assert!(t.notes.iter().all(|s| !s.starts_with("tabs:")), "{:?}", t.notes);
}

#[test]
fn a_tab_that_a_second_ring_cuts_into_holds_the_loop() {
    // Two parts 10 mm apart: each ring passes 4 mm from a tab of the other
    // one and cut it off (a neck of 0), with no note. Both loops are held.
    let both = pair(10.0);
    let t = contour(&both, &params(4)).unwrap();
    assert_eq!(held(&t), 2, "{:?} necks {:?}", t.notes, necks(&both, &t));
    assert!(cuts_nothing_at_the_tabs(&t), "held loops cut nothing under the tab top");
    // 14 mm apart: 2 mm of stock between the kerfs, named for each loop.
    let both = pair(14.0);
    let t = contour(&both, &params(4)).unwrap();
    assert_eq!(held(&t), 0, "{:?}", t.notes);
    let named = t.notes.iter().filter(|s| s.starts_with("tabs:") && s.contains("leaves only 2.000 mm of material beside it")).count();
    assert_eq!(named, 2, "{:?}", t.notes);
    assert_eq!(necks(&both, &t).len(), 8);
}
