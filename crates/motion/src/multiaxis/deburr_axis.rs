//! Multi-axis Deburr (#1894): the 3D Deburr's edge break with the
//! tool axis tilted to the break itself, so steep and vertical edges,
//! which a vertical cone cannot reach, break the same as level ones.
//!
//! The edges, the filters and the break size are the 3D Deburr's
//! (`finishing3d::deburr_scan`, `deburr_filter`, `DeburrBreak`). The
//! pose is `deburr_pose`'s: one cone flank lies on the chamfer plane,
//! and the axis turns about the plane's normal - the one freedom that
//! leaves the cut the same. On it the ride takes the nominal lean
//! (`lead - lag`); where the shank or holder would touch the part
//! (`avoid::shank_hits_at`, the stage F test), it takes the clear pose
//! nearest the previous one inside `max_tilt_deg`, so the axis moves
//! as little as it can. An edge segment that no admitted pose reaches
//! follows the 3D Deburr's unreachable-edge policy: skipped with a
//! note, or the whole ride refused by name. A turn of more than
//! `LINK_TURN_DEG` between two segments ends the run, and the tool
//! leaves along its axis and comes back in the air.
//!
//! Provenance: tilted chamfer-mill deburring and tool-axis tilt
//! against holder collision are published 5-axis practice (Gosselin
//! 2021 on holder-aware tilt; machine-tool handbook chamfer geometry).
//! Our own design notes; no vendor material consulted.

use std::collections::{BTreeMap, HashMap};

use crate::cl::{BreakFrame, EdgeBreak, EdgeBreakVolume, MoveKind, PathBuilder, Toolpath};
use crate::finishing3d::deburr_filter::{DeburrEdges, EdgeFilter};
use crate::finishing3d::deburr_scan::{scan, EdgeScan, Exposed};
use crate::finishing3d::deburr_size::clip;
use crate::finishing3d::{crease, DeburrParams, UnreachableEdges};
use crate::geometry::{Point3, Triangle};
use crate::machining2d::GenError;

use super::avoid::{shank_hits_at, Shank};
use super::deburr_pose::{add, ang, cross, dot, scale, sub, unit, Cross, DeburrAxisMode, Lean, Pose};
use super::stamp_link::LINK_TURN_DEG;

/// The largest tilt cap, degrees: the escape cone of stage F.
pub const MAX_TILT_CAP_DEG: f64 = 85.0;
/// The most posed stretches one edge segment is cut into.
const MAX_STRETCHES: usize = 256;

/// One multi-axis Deburr ride's numbers.
#[derive(Clone, Copy, Debug)]
pub struct AxisDeburrParams {
    /// The 3D Deburr's numbers: width, cone, window, feeds, filters,
    /// the unreachable-edge policy and the break size.
    pub base: DeburrParams,
    pub mode: DeburrAxisMode,
    /// Forward lean of the axis along the travel, degrees.
    pub lead_deg: f64,
    /// Backward lean of the axis along the travel, degrees.
    pub lag_deg: f64,
    /// The most the axis may turn off the edge's natural pose, degrees.
    pub max_tilt_deg: f64,
    /// The cone's largest diameter, mm: it sets the flank length.
    pub tool_diameter: f64,
    /// The shank and holder to keep off the part; `None` tests nothing.
    pub shank: Option<Shank>,
}

impl AxisDeburrParams {
    fn validate(&self, facets: &[Triangle]) -> Result<(), GenError> {
        self.base.validate(facets)?;
        let fin = [self.lead_deg, self.lag_deg, self.max_tilt_deg, self.tool_diameter];
        if fin.iter().any(|v| !v.is_finite()) {
            return Err(GenError::Invalid("multi-axis deburr angles and tool diameter must be finite".into()));
        }
        if self.tool_diameter <= 0.0 {
            return Err(GenError::Invalid("multi-axis deburr needs a tool diameter above zero".into()));
        }
        if !(0.0..=MAX_TILT_CAP_DEG).contains(&self.max_tilt_deg) {
            return Err(GenError::Invalid(format!(
                "multi-axis deburr max tilt must be 0 to {MAX_TILT_CAP_DEG} deg, not {}",
                self.max_tilt_deg
            )));
        }
        for (name, v) in [("lead", self.lead_deg), ("lag", self.lag_deg)] {
            if !(0.0..=MAX_TILT_CAP_DEG).contains(&v) {
                return Err(GenError::Invalid(format!("multi-axis deburr {name} must be 0 to {MAX_TILT_CAP_DEG} deg, not {v}")));
            }
        }
        if (self.lead_deg - self.lag_deg).abs() > self.max_tilt_deg {
            return Err(GenError::Invalid(format!(
                "multi-axis deburr lead minus lag ({} deg) is more than the max tilt ({} deg) - raise Max tilt or lower the lean",
                self.lead_deg - self.lag_deg,
                self.max_tilt_deg
            )));
        }
        Ok(())
    }
}

/// The kept edge segments by welded endpoint pair: plane and setback.
type Book = HashMap<((i64, i64, i64), (i64, i64, i64)), (Cross, f64)>;

/// One posed edge segment of a run: edge points, plane, setback, pose.
#[derive(Clone, Copy)]
struct Station {
    p: Point3,
    q: Point3,
    cr: Cross,
    s: f64,
    pose: Pose,
}

/// Break the exposed edges of `facets` with a tilted cone. See the
/// module notes for the laws.
pub fn multiaxis_deburr(facets: &[Triangle], p: &AxisDeburrParams, edges: &DeburrEdges<'_>) -> Result<Toolpath, GenError> {
    p.validate(facets)?;
    let b0 = &p.base;
    let half = 0.5 * b0.tip_angle_deg.to_radians();
    let lean = Lean {
        mode: p.mode,
        half,
        flank: 0.5 * p.tool_diameter / half.sin(),
        lean: (p.lead_deg - p.lag_deg).to_radians(),
        max_tilt_deg: p.max_tilt_deg,
        rotary: Point3::new(1.0, 0.0, 0.0),
    };
    let w = crease::vkey_scale(facets);
    let mut filter = EdgeFilter::new(facets, b0, edges, w)?;
    let EdgeScan { edges: exposed, flips, manifold } = scan(facets, w);

    // Keep the edges in the window, sized; group them by setback so a
    // run keeps one break (the 3D Deburr's grouping by drop).
    let mut book: Book = HashMap::new();
    let mut groups: BTreeMap<i64, Vec<crease::Seg>> = BTreeMap::new();
    let (mut out_of_size, mut degenerate) = (0usize, 0usize);
    for (e0, e1, kind) in &exposed {
        let bend = match kind {
            Exposed::Rim { .. } => None,
            Exposed::Crease { bend, .. } => Some(*bend),
        };
        if !filter.keeps(*e0, *e1, bend) || e0.z.max(e1.z) <= b0.bottom_z {
            continue;
        }
        let Some(cr) = cross(*e0, *e1, kind, facets) else {
            degenerate += 1;
            continue;
        };
        let Some(s) = b0.sizing.setback(b0.width, cr.wedge_deg) else {
            out_of_size += 1;
            continue;
        };
        let Some((q0, q1)) = clip(*e0, *e1, b0, 0.0) else { continue };
        let (k0, k1) = (crease::vkey(q0, w), crease::vkey(q1, w));
        book.insert(pair(k0, k1), (cr, s));
        groups.entry((s * 1e9).round() as i64).or_default().push(crease::Seg { c0: q0, c1: q1, k0, k1 });
    }
    let mut runs: Vec<Vec<Point3>> = groups.values().flat_map(|g| crease::chain(g)).collect();
    runs.sort_by(|a, b| crease::cmp_pt(a[0], b[0]));

    let r = 0.5 * p.tool_diameter;
    let boxes: Vec<[f64; 6]> = facets.iter().map(aabb).collect();
    let clear = |st: &Station, pose: &Pose| -> bool {
        let Some(sh) = &p.shank else { return true };
        let len = dot(sub(st.q, st.p), sub(st.q, st.p)).sqrt();
        let n = ((len / r.max(1e-3)).ceil() as usize).clamp(1, 64);
        b0.sizing.pass_setbacks(st.s).all(|sk| {
            (0..=n).all(|i| {
                let e = add(st.p, scale(sub(st.q, st.p), i as f64 / n as f64));
                let tip = lean.tip(&st.cr, pose, e, sk);
                let c = add(tip, scale(Point3::new(pose.a[0], pose.a[1], pose.a[2]), r));
                !shank_hits_at(c, pose.a, r, sh, facets, &boxes)
            })
        })
    };

    // Pose every segment; cut a run where a segment is unreachable or
    // the axis would swing more than the link turn at the contact.
    let (mut no_pose, mut holder) = (0usize, 0usize);
    let mut pieces: Vec<Vec<Station>> = Vec::new();
    for run in &runs {
        let mut cur: Vec<Station> = Vec::new();
        let mut prev: Option<[f64; 3]> = None;
        for (probe, t) in stretches(run, &book, w, p.tool_diameter, &lean) {
            let cands = lean.candidates(&probe.cr, t, probe.cr.chord(probe.s));
            let pick = match cands.first() {
                None => None,
                Some(first) if clear(&probe, first) => Some(*first),
                Some(_) => {
                    let mut rest = cands[1..].to_vec();
                    let toward = prev.unwrap_or(cands[0].a);
                    rest.sort_by(|x, y| ang(x.a, toward).total_cmp(&ang(y.a, toward)));
                    let got = rest.into_iter().find(|c| clear(&probe, c));
                    holder += usize::from(got.is_none());
                    got
                }
            };
            no_pose += usize::from(cands.is_empty());
            let Some(pose) = pick else {
                pieces.extend((!cur.is_empty()).then(|| std::mem::take(&mut cur)));
                prev = None;
                continue;
            };
            if prev.is_some_and(|pa| ang(pa, pose.a) > LINK_TURN_DEG) {
                pieces.push(std::mem::take(&mut cur));
            }
            prev = Some(pose.a);
            cur.push(Station { pose, ..probe });
        }
        pieces.extend((!cur.is_empty()).then_some(cur));
    }

    let unreachable = no_pose + holder;
    if unreachable > 0 && b0.unreachable == UnreachableEdges::Refuse {
        return Err(GenError::Invalid(format!(
            "multi-axis deburr: {unreachable} edge segments have no clear pose ({no_pose} outside the max tilt, the rotary plane or the cone's flank; {holder} blocked by the shank or holder), and Unreachable edges is set to Refuse; raise Max tilt, use Axis5, or set it to Skip with note"
        )));
    }
    if pieces.is_empty() {
        if let Some(why) = filter.empty() {
            return Err(why);
        }
        return Err(GenError::Empty(format!(
            "multi-axis deburr found no edge it can break inside the programmed window ({unreachable} edge segments have no clear pose) - an exposed edge is an open mesh's rim or a convex crease"
        )));
    }

    let mut b = PathBuilder::approached(Point3::new(0.0, 0.0, b0.clearance_z));
    for (n, what) in [
        (flips, "shared edges skipped for inconsistent winding"),
        (manifold, "edges touch more than two facets and were skipped"),
        (degenerate, "edges have degenerate faces and were skipped"),
        (out_of_size, "edges have a wedge angle outside the break size's range and were skipped"),
        (no_pose, "edge segments have no pose inside the max tilt, the rotary plane or the cone's flank and were skipped"),
        (holder, "edge segments where every pose inside the max tilt touches the shank or holder were skipped"),
    ] {
        if n > 0 {
            b.note(format!("multi-axis deburr: {n} {what}"));
        }
    }
    for note in filter.notes() {
        b.note(note);
    }
    for piece in &pieces {
        emit(&mut b, piece, &lean, b0)?;
    }
    let mut path = b.finish();
    if !path.all_finite() {
        return Err(GenError::Invalid("multi-axis deburr path left the finite horizon".into()));
    }
    path.breaks = pieces.iter().filter_map(|pc| declared(pc)).filter(EdgeBreak::is_sane).collect();
    Ok(path)
}

/// The run's stretches in travel order, each with its travel tangent
/// and its edge's plane and setback (the pose left natural). A stretch
/// is one tool diameter or less, so the axis can turn near a local
/// obstacle and back on one long edge segment.
fn stretches(run: &[Point3], book: &Book, w: f64, step: f64, lean: &Lean) -> Vec<(Station, Point3)> {
    let mut out = Vec::new();
    for wnd in run.windows(2) {
        let (a0, b1) = (wnd[0], wnd[1]);
        let (Some(&(cr, s)), Some(t)) = (book.get(&pair(crease::vkey(a0, w), crease::vkey(b1, w))), unit(sub(b1, a0))) else {
            continue;
        };
        let len = dot(sub(b1, a0), sub(b1, a0)).sqrt();
        let k = ((len / step).ceil() as usize).clamp(1, MAX_STRETCHES);
        let at = |i: usize| if i == k { b1 } else { add(a0, scale(sub(b1, a0), i as f64 / k as f64)) };
        for i in 0..k {
            out.push((Station { p: at(i), q: at(i + 1), cr, s, pose: lean.natural(&cr, t) }, t));
        }
    }
    out
}

/// Ride one posed piece once for each pass: in along the first axis,
/// along the edge (a turn at a vertex is a feed at the contact, less
/// than the link turn), and out along the last axis.
fn emit(b: &mut PathBuilder, piece: &[Station], lean: &Lean, p: &DeburrParams) -> Result<(), GenError> {
    let (Some(first), Some(last)) = (piece.first(), piece.last()) else { return Ok(()) };
    for sk in p.sizing.pass_setbacks(first.s) {
        let t0 = lean.tip(&first.cr, &first.pose, first.p, sk);
        b.set_axis(Some(first.pose.a));
        b.push(MoveKind::Rapid, above(t0, first.pose.a, p.clearance_z)?, None);
        b.push(MoveKind::Plunge, t0, Some(p.plunge_feed));
        for st in piece {
            b.set_axis(Some(st.pose.a));
            let start = lean.tip(&st.cr, &st.pose, st.p, sk);
            let at = b.at();
            if dot(sub(start, at), sub(start, at)) > 1e-18 {
                b.push(MoveKind::Feed, start, Some(p.feed));
            }
            b.push(MoveKind::Feed, lean.tip(&st.cr, &st.pose, st.q, sk), Some(p.feed));
        }
        let end = b.at();
        b.push(MoveKind::Rapid, above(end, last.pose.a, p.clearance_z)?, None);
    }
    Ok(())
}

/// The point on the axis line through `tip` at the clearance height:
/// the tool enters and leaves along its own axis, which only moves it
/// further off the chamfer plane. A tip above the clearance stays put.
fn above(tip: Point3, a: [f64; 3], clearance_z: f64) -> Result<Point3, GenError> {
    let d = (clearance_z - tip.z).max(0.0) / a[2];
    let out = add(tip, scale(Point3::new(a[0], a[1], a[2]), d));
    if out.is_finite() {
        Ok(out)
    } else {
        Err(GenError::Invalid("multi-axis deburr: an axis is too flat to leave to the clearance plane".into()))
    }
}

/// The break one piece was asked to cut: a framed flat cap, the
/// chamfer plane's normal outward and the chord across (#1891).
fn declared(piece: &[Station]) -> Option<EdgeBreak> {
    let first = piece.first()?;
    let mut edge = vec![first.p];
    edge.extend(piece.iter().map(|st| st.q));
    let frames = piece
        .iter()
        .map(|st| BreakFrame { outward: [st.cr.n.x, st.cr.n.y, st.cr.n.z], across: [st.cr.c.x, st.cr.c.y, st.cr.c.z] })
        .collect();
    Some(EdgeBreak {
        edge,
        depth: first.cr.depth(first.s),
        slope: 0.0,
        volume: Some(EdgeBreakVolume { width: first.cr.chord(first.s), frames, profile: crate::cl::BreakProfile::Flat }),
    })
}

fn pair<K: Ord>(a: K, b: K) -> (K, K) {
    if a <= b { (a, b) } else { (b, a) }
}

fn aabb(f: &Triangle) -> [f64; 6] {
    [
        f.a.x.min(f.b.x.min(f.c.x)),
        f.a.y.min(f.b.y.min(f.c.y)),
        f.a.z.min(f.b.z.min(f.c.z)),
        f.a.x.max(f.b.x.max(f.c.x)),
        f.a.y.max(f.b.y.max(f.c.y)),
        f.a.z.max(f.b.z.max(f.c.z)),
    ]
}
