//! The pcurves the STEP reader has no closed form for (a free-form edge on a
//! spline face, #917), made by the kernel itself: the face goes to the kernel
//! alone as an open one-face sheet, the uses without a pcurve go bare, and
//! the kernel makes their UV trims as it makes every trim it needs.
//! Provenance: the maintainer's Solidcraft `SmlibTrims`
//! (`crates/smlib/src/pcurves.rs`, reused with his permission), moved onto
//! the solidcore `Trims` seam (solidcore-ops `pcurve::Trims`, solidcore-step
//! `topology::read_solids_with`) and solidcore-convert's record/kernel
//! conversion, at the commit pinned in Cargo.toml. No HSMWorks material was
//! consulted.
//!
//! Nothing is accepted unchecked (Prime Directive): an edge farther off its
//! surface than the reader's reach, an edge interval the kernel moved past
//! the edge's tolerance, and a made pcurve whose lift onto the surface
//! misses its 3D edge by more than the edge's tolerance are each refused by
//! name, and the reader then refuses the solid. The reader checks the record
//! again after this (`record_check::agreement`).

use std::cell::RefCell;

use solidcore::Kernel;
use solidcore::eval::{CurveEval, SurfaceEval};
use solidcore::geom::tol::{SmallTopoType, context_xsect_tol_3d};
use solidcore::heal::gap::GapFunction;
use solidcore_convert::brep::{Order, to_kernel_with_bare_uses, to_record};
use solidcore_ops::pcurve::{SURFACE_GAP_MARGIN, TrimEdge, TrimFace, Trims};
use solidcore_record::brep::{
    self as b, Curve2, EdgeUseRef, Loop, OrientedEdge, OrientedFace, Region, Shell, Vertex,
};

/// The farthest an edge may lie off its face's surface and still be kept
/// (mm): the reader's own reach (its `MAX_TOLERANCE`, "a vertex or edge
/// lies off its curve or surface by more than 1 mm" refuses the body).
pub const MAX_EDGE_GAP_MM: f64 = 1.0;

/// Samples of the gap function along one edge (the kernel's own default).
const GAP_SAMPLES: usize = 20;

/// Samples of a made pcurve's lift checked against its 3D edge, ends
/// included (one more than this).
const LIFT_SAMPLES: usize = 32;

/// [`Trims`] by the kernel of the seam: the solidcore kernel the import runs
/// on, so its tolerances, opt-in fixes and time bound hold here too.
pub struct KernelTrims<'k> {
    k: RefCell<&'k mut Kernel>,
}

impl<'k> KernelTrims<'k> {
    /// Trims made on `k`.
    pub fn new(k: &'k mut Kernel) -> Self {
        Self { k: RefCell::new(k) }
    }
}

impl Trims for KernelTrims<'_> {
    fn trims(&self, face: &TrimFace<'_>) -> Result<Vec<Vec<Curve2>>, String> {
        let mut k = self
            .k
            .try_borrow_mut()
            .map_err(|_| "kernel pcurves: the kernel is already in use".to_string())?;
        trims(&mut k, face).map_err(|e| format!("kernel pcurves: {e}"))
    }
}

fn trims(k: &mut Kernel, face: &TrimFace<'_>) -> Result<Vec<Vec<Curve2>>, String> {
    let (mut record, bare) = sheet(face)?;
    let build = |k: &mut Kernel, r: &b::Brep| {
        to_kernel_with_bare_uses(k, r, true, &bare).map_err(|e| e.to_string())
    };
    let (mut body, mut order) = build(k, &record)?;
    if cover_gaps(k, &body, &order, face, &bare, &mut record)? {
        (body, order) = build(k, &record)?;
    }
    let (out, back) = to_record(k, &mut body).map_err(|e| e.to_string())?;
    let [made] = out.faces.as_slice() else {
        return Err(format!("the kernel made {} faces of one", out.faces.len()));
    };
    let made_uses: Vec<&OrientedEdge> = made
        .loops
        .iter()
        .flat_map(|l| match l {
            Loop::Edges { edges } => edges.as_slice(),
            Loop::Vertex { .. } => &[],
        })
        .collect();
    // Our edge `i` is the kernel's `order.edges[i]`, which the record read
    // back puts where `back.edges` names it.
    let edge_of = |i: u32| -> Result<usize, String> {
        let id = order.edges.get(i as usize).ok_or("an edge the kernel lost")?;
        back.edges
            .iter()
            .position(|x| x == id)
            .ok_or_else(|| "an edge the kernel lost".to_string())
    };
    // A sheet's loops come back the way the kernel runs them, which may be
    // against ours: a use is the same pcurve either way (on its edge's
    // interval in its curve's direction), and the loop's way round is read
    // from an edge the face uses once.
    let ours = face.loops.iter().flatten();
    let once = ours
        .clone()
        .find(|u| ours.clone().filter(|v| v.edge == u.edge).count() == 1)
        .ok_or("a face of seam edges only")?;
    let at = edge_of(once.edge)?;
    let reversed = made_uses
        .iter()
        .find(|m| m.edge as usize == at)
        .ok_or("a use the kernel did not keep")?
        .forward
        != once.forward;
    let mut result = Vec::with_capacity(face.loops.len());
    for l in &face.loops {
        let mut pcs = Vec::with_capacity(l.len());
        for u in l {
            let e = edge_of(u.edge)?;
            let edge = out.edges.get(e).ok_or("an edge the kernel lost")?;
            let ours = face.edges.get(u.edge as usize).ok_or("edge reference")?;
            let found = made_uses
                .iter()
                .find(|m| m.edge as usize == e && m.forward == (u.forward != reversed));
            let pc = found.ok_or("a use the kernel did not keep")?.pcurve.clone();
            // The edge's tolerance as the kernel held it (raised where the
            // edge lies off the surface, `cover_gaps`).
            let held = record
                .edges
                .get(u.edge as usize)
                .map_or(ours.tolerance, |x| x.tolerance)
                .max(ours.tolerance);
            let reach = ours.vertices.map(|v| {
                face.vertex_tolerances
                    .get(v as usize)
                    .copied()
                    .unwrap_or(0.0)
                    .max(held)
            });
            let pc = on_our_interval(pc, ours, edge.interval, reach)?;
            lifts_onto(face, ours, &pc, held)?;
            pcs.push(pc);
        }
        result.push(pcs);
    }
    Ok(result)
}

/// Each bare use's edge measured against the face's surface as the kernel
/// measures it (over the surface's own domain), and its tolerance in
/// `record` raised to cover that gap with [`SURFACE_GAP_MARGIN`]: the kernel
/// keeps an edge whole when it makes its trim only when its tolerance covers
/// the gap, and otherwise trims the edge back to the patch. Whether any was
/// raised; an edge farther off than [`MAX_EDGE_GAP_MM`] is refused with how
/// far.
fn cover_gaps(
    k: &Kernel,
    body: &solidcore::Brep,
    order: &Order,
    face: &TrimFace<'_>,
    bare: &[(u32, u32, u32)],
    record: &mut b::Brep,
) -> Result<bool, String> {
    let mut edges: Vec<usize> = Vec::new();
    for (li, l) in face.loops.iter().enumerate() {
        for (p, u) in l.iter().enumerate() {
            let (Ok(li), Ok(p)) = (u32::try_from(li), u32::try_from(p)) else {
                return Err("too many uses".into());
            };
            if bare.contains(&(0, li, p)) && !edges.contains(&(u.edge as usize)) {
                edges.push(u.edge as usize);
            }
        }
    }
    let f = *order.faces.first().ok_or("the face the kernel lost")?;
    let surface = body.face(f).map_err(|e| e.to_string())?.surface.as_ref();
    let surface = surface.ok_or("a face without a surface")?;
    let tol = context_xsect_tol_3d(k, None, SmallTopoType::NormalSize);
    let mut raised = false;
    for e in edges {
        let id = *order.edges.get(e).ok_or("an edge the kernel lost")?;
        let edge = body.edge(id).map_err(|x| x.to_string())?;
        let curve = edge.curve.as_ref().ok_or("an edge without a curve")?;
        if !(edge.interval.max() > edge.interval.min()) {
            return Err("an empty edge interval".into());
        }
        let mut gaps = GapFunction::crv_srf(tol, curve, &edge.interval, surface, GAP_SAMPLES);
        let gap = gaps
            .max_gap_sample(k, false)
            .map_err(|x| x.to_string())?
            .length();
        if !gap.is_finite() {
            return Err("an edge whose gap to the surface is not finite".into());
        }
        let need = SURFACE_GAP_MARGIN * gap;
        let given = record.edges.get_mut(e).ok_or("edge reference")?;
        if need <= given.tolerance {
            continue;
        }
        if !(need <= MAX_EDGE_GAP_MM) {
            return Err(format!("an edge lies {gap} mm off the face's surface"));
        }
        given.tolerance = need;
        // A vertex's tolerance covers every edge that meets it.
        for v in given.vertices {
            if let Some(x) = record.vertices.get_mut(v as usize) {
                x.tolerance = x.tolerance.max(need);
            }
        }
        raised = true;
    }
    Ok(raised)
}

/// A pcurve the kernel made on its edge's interval `theirs`, on ours. The
/// kernel may end an edge at a parameter of its own (a knot its vertex lies
/// at, the end of the part on the face): where each end it chose is within
/// `reach` of ours on the edge's curve (that end's vertex tolerance, or the
/// edge's tolerance as the kernel held it where larger), the pcurve is mapped
/// onto our interval by the affine change of parameter (exact for a
/// B-spline: its knots move). Farther is the kernel trimming the edge,
/// refused with how far.
pub(crate) fn on_our_interval(
    pc: Curve2,
    ours: &TrimEdge<'_>,
    theirs: [f64; 2],
    reach: [f64; 2],
) -> Result<Curve2, String> {
    let [a, bb] = ours.interval;
    let [c, d] = theirs;
    if a == c && bb == d {
        return Ok(pc);
    }
    let curve = edge_eval(ours)?;
    let at = |t: f64| {
        curve
            .point(t)
            .ok_or_else(|| format!("the edge's curve has no point at {t}"))
    };
    let shift = [at(a)?.distance(at(c)?), at(bb)?.distance(at(d)?)];
    if !(shift[0] <= reach[0] && shift[1] <= reach[1]) || !(d > c) {
        return Err(format!(
            "the kernel moved an edge's interval from {:?} to {theirs:?}, its ends {shift:?} mm along its curve (tolerance {reach:?} mm)",
            ours.interval
        ));
    }
    let map = |x: f64| {
        if x == c {
            a
        } else if x == d {
            bb
        } else {
            a + (x - c) * ((bb - a) / (d - c))
        }
    };
    Ok(match pc {
        Curve2::Spline { knots, points } => Curve2::Spline {
            knots: knots.into_iter().map(map).collect(),
            points,
        },
        Curve2::RationalSpline { knots, homogeneous } => Curve2::RationalSpline {
            knots: knots.into_iter().map(map).collect(),
            homogeneous,
        },
    })
}

/// The lift check: `pc` taken onto the face's surface lies within
/// `tolerance` of its 3D edge at every sample of the edge's interval, ends
/// included. A made pcurve that misses is refused with how far - never a
/// face trimmed along a curve that is not its edge.
fn lifts_onto(
    face: &TrimFace<'_>,
    ours: &TrimEdge<'_>,
    pc: &Curve2,
    tolerance: f64,
) -> Result<(), String> {
    let surface = solidcore_convert::surface(face.nurbs.unwrap_or(face.surface))
        .map_err(|e| e.to_string())?;
    let on = SurfaceEval::new(&surface).map_err(|e| e.to_string())?;
    let curve = edge_eval(ours)?;
    let [t0, t1] = ours.interval;
    let mut gap = 0.0_f64;
    for i in 0..=LIFT_SAMPLES {
        let t = if i == LIFT_SAMPLES {
            t1
        } else {
            t0 + (t1 - t0) * (i as f64 / LIFT_SAMPLES as f64)
        };
        let q = solidcore_ops::pcurve::eval(pc, t)
            .ok_or_else(|| format!("the made pcurve has no point at {t}"))?;
        let s = on
            .point(q)
            .ok_or_else(|| format!("the surface has no point at {q:?}"))?;
        let p = curve
            .point(t)
            .ok_or_else(|| format!("the edge's curve has no point at {t}"))?;
        gap = gap.max(s.distance(p));
    }
    if !(gap <= tolerance) {
        return Err(format!(
            "a made pcurve lies {gap} mm off its edge on the surface (tolerance {tolerance} mm)"
        ));
    }
    Ok(())
}

/// The evaluator of the curve an edge is parameterised on.
fn edge_eval(ours: &TrimEdge<'_>) -> Result<CurveEval, String> {
    let curve =
        solidcore_convert::curve(ours.nurbs.unwrap_or(ours.curve)).map_err(|e| e.to_string())?;
    CurveEval::new(&curve).map_err(|e| e.to_string())
}

/// The face as a one-face open sheet, and its uses that go without trims
/// (each `(face, loop, position)`, in order, so sorted).
fn sheet(face: &TrimFace<'_>) -> Result<(b::Brep, Vec<(u32, u32, u32)>), String> {
    let mut edges: Vec<b::Edge> = face
        .edges
        .iter()
        .map(|e| b::Edge {
            curve: e.curve.clone(),
            nurbs: e.nurbs.cloned(),
            interval: e.interval,
            vertices: e.vertices,
            tolerance: e.tolerance,
            uses: Vec::new(),
        })
        .collect();
    let mut bare = Vec::new();
    let mut loops = Vec::with_capacity(face.loops.len());
    for (li, l) in face.loops.iter().enumerate() {
        let li = u32::try_from(li).map_err(|_| "too many loops")?;
        let mut uses = Vec::with_capacity(l.len());
        for (p, u) in l.iter().enumerate() {
            let p = u32::try_from(p).map_err(|_| "too many uses")?;
            let edge = edges.get_mut(u.edge as usize).ok_or("edge reference")?;
            edge.uses.push(EdgeUseRef {
                face: 0,
                r#loop: li,
                position: p,
                upper: !u.forward,
            });
            let pcurve = match &u.pcurve {
                Some(pc) => pc.clone(),
                None => {
                    bare.push((0, li, p));
                    // Not read: the use goes without a trim.
                    let [s, e] = edge.interval;
                    Curve2::Spline {
                        knots: vec![s, s, e, e],
                        points: vec![[0.0; 2]; 2],
                    }
                }
            };
            uses.push(OrientedEdge {
                edge: u.edge,
                forward: u.forward,
                pcurve,
            });
        }
        loops.push(Loop::Edges { edges: uses });
    }
    let vertices = face
        .vertices
        .iter()
        .enumerate()
        .map(|(i, &point)| Vertex {
            point,
            tolerance: face
                .vertex_tolerances
                .get(i)
                .copied()
                .unwrap_or(0.0)
                .max(face.tolerance),
        })
        .collect();
    let record = b::Brep {
        version: b::VERSION,
        tolerance: face.tolerance,
        regions: vec![Region {
            shells: vec![0],
            void: false,
        }],
        shells: vec![Shell::Faces {
            faces: vec![OrientedFace {
                face: 0,
                forward: true,
            }],
            wires: Vec::new(),
        }],
        faces: vec![b::Face {
            surface: face.surface.clone(),
            nurbs: face.nurbs.cloned(),
            domain: face.domain,
            loops,
        }],
        edges,
        vertices,
    };
    Ok((record, bare))
}
