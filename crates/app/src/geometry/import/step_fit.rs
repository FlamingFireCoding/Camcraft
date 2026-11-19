//! A curved STEP face's facets held to the face's own surface (#2600): each
//! facet edge inside the face is measured against the surface, and an edge
//! that stands off it is split at the surface until it does not.
//!
//! Provenance: the chord height (sagitta) of a facet edge, measured from the
//! edge's midpoint to the surface point at the midpoint of its parameters
//! (elementary geometry), and edge-midpoint bisection with the red-green
//! split patterns of adaptive mesh refinement (Bank, Sherman and Weiser,
//! "Refinement algorithms and data structures for regular local mesh
//! refinement", 1983; Rivara, "Algorithms for refining triangular grids
//! suitable for adaptive and multigrid techniques", 1984). The mesh it
//! measures is truck's (ricosjp, Apache-2.0). Our own code; nothing from any
//! CAM vendor.
//!
//! Why: truck meshes a trimmed face by a constrained Delaunay triangulation
//! of its parameter plane, over the boundary's points and the points of a
//! parameter grid that lie inside the boundary. A cylinder is straight along
//! its axis, so its grid has two rows, and both are on the boundary: no grid
//! point is inside. A plain wall still meshes well (rim to rim). A wall with
//! a hole in it does not: the empty-circle rule joins each rim edge to the
//! hole's rim, a quarter turn away, and the facet runs through the solid. A
//! D50 bar with two cross holes came in with its wall up to 25 mm inside the
//! true cylinder, and the importer reported the tolerance it had asked for.
//! Every gouge law then judged cuts against a part that was not there.
//!
//! How: after a pass, each curved elementary face (cylinder, cone, sphere,
//! torus - the kinds that take truck's own grid and are never stretched) is
//! read as the planar triangulation it is in (u, v). An edge two facets of
//! the face share is INSIDE the face; its chord height is measured, and one
//! over the pass tolerance (divided by [`Curved::inside`]) is split at the
//! surface. The
//! split is by the edge alone, so the two facets that share it split alike
//! and the face stays one conforming mesh. An edge only one facet has is on
//! the face's boundary, which the neighbouring face meshes too: it is never
//! split, so the shell stays closed. What cannot be refined (the facet
//! budget, the round cap) is REPORTED as the mesh's chord error, never
//! dropped: the model then carries reduced fidelity (#864) and the gate
//! refuses to verify against it.
//!
//! A refined face then gets its flat "cap" facets flipped away ([`relax`],
//! #2713): the split holds the facets' points and edges to the surface, and
//! the flip holds their normals to it.
//!
//! What is reported (#2718): the largest height of EVERY edge of the kept
//! facets (inside and boundary, under the limit or not), times a bound for
//! the inside of a facet. Provenance of the bound: the error of linear
//! interpolation of a quadratic over a triangle is
//! `1/2 * sum(l_i * l_j * q_ij)` over its edges (barycentric `l`, `q_ij`
//! the edge's second difference; Waldron, "The error in linear
//! interpolation at the vertices of a simplex", SIAM J. Numer. Anal. 35,
//! 1998). An edge midpoint carries `q/8`, and `sum(l_i * l_j)` is at most
//! 1/3, so no inside point stands more than 4/3 of the highest edge off a
//! surface curved two ways (sphere, torus; on a sphere the equilateral
//! facet reaches it). A cylinder is curved one way: its facet projects to
//! a triangle inscribed in the circle, whose point nearest the axis is on
//! an edge, so its edges bound it exactly. A cone is curved one way too,
//! but its parameter midpoint is not the edge's nearest surface point; it
//! takes the two-way bound. [`Curved::inside`] adds a margin over 4/3 for
//! the higher-order terms a facet of real size carries (on a sphere the
//! equilateral ratio is 1.36 at a 30-degree facet).
//!
//! Limits, stated: B-spline, NURBS and swept faces are not measured here
//! (their grid and parameter stretch are #1602's); a face whose mesh does
//! not carry usable parameters is left as the kernel made it.

use std::collections::HashMap;

use truck_meshalgo::prelude::*;
use truck_stepio::r#in::alias::{ElementarySurface, ParametricSurface, Surface};

use super::step_guard::Metered;
use crate::geometry::{Point3 as P3, Triangle};

/// Refinement rounds. Each halves an edge's parameter span, so its chord
/// height falls to a quarter: 24 rounds take any span to round-off.
const ROUNDS: usize = 24;

/// Flips of [`relax`] for each facet of the face. A flip can make its
/// neighbour edges due; the cap bounds the work on a mesh that never rests.
const FLIPS_PER_FACET: usize = 8;
/// An edge is flipped when its two opposite angles sum to more than a half
/// turn plus this (radians), so a square cell's two equal diagonals stay.
const FLIP_MARGIN: f64 = 1e-3;

/// One face's verdict: the facets that replace the kernel's (`None`: keep
/// them), `off`: how far the kept facets may stand off the surface (the
/// highest measured edge times the kind's inside bound, #2718; 0 for a
/// face not measured), and `left`: the highest inside edge still over the
/// limit (0 when none is).
pub(super) struct Fit {
    pub facets: Option<Vec<Triangle>>,
    pub off: f64,
    pub left: f64,
}

const KEPT: Fit = Fit { facets: None, off: 0.0, left: 0.0 };

/// The curved elementary kinds measured here.
#[derive(Clone, Copy, Debug)]
pub(super) enum Curved {
    Cylinder,
    Cone,
    Sphere,
    Torus,
}

impl Curved {
    /// How far the inside of a facet may stand off the surface, as a
    /// multiple of its highest edge (see the module header).
    pub(super) fn inside(self) -> f64 {
        match self {
            Curved::Cylinder => 1.0,
            Curved::Cone | Curved::Sphere | Curved::Torus => 1.5,
        }
    }
}

/// [`fit`] for face `surface` of a shell, when its kind is measured here.
pub(super) fn fit_face(surface: Option<&Metered>, poly: &PolygonMesh, tol: f64, room: usize) -> Fit {
    let Some(s) = surface else { return KEPT };
    let kind = match &s.inner {
        Surface::ElementarySurface(e) => match **e {
            ElementarySurface::Plane(_) => return KEPT,
            ElementarySurface::CylindricalSurface(_) => Curved::Cylinder,
            ElementarySurface::ConicalSurface(_) => Curved::Cone,
            ElementarySurface::Sphere(_) => Curved::Sphere,
            ElementarySurface::ToroidalSurface(_) => Curved::Torus,
        },
        _ => return KEPT,
    };
    // Not through the meter: this is a measure of the finished pass.
    let at = |u: f64, v: f64| {
        let (u, v) = s.own(u, v);
        let p = s.inner.subs(u, v);
        [p.x, p.y, p.z]
    };
    let fit = fit(poly, &at, tol, room, kind);
    if fit.left > 0.0 {
        tracing::warn!("step: a curved face's facets stand {:e} off its surface at tol {tol:e} and could not be refined", fit.left);
    }
    fit
}

struct Vert {
    uv: [f64; 2],
    p: [f64; 3],
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

/// Hold `poly`'s inside edges within `tol / kind.inside()` of the surface `at`
/// evaluates in the mesh's own parameters, making at most `room` facets,
/// and bound how far the kept facets stand off a surface of kind `kind`.
pub(super) fn fit(poly: &PolygonMesh, at: &dyn Fn(f64, f64) -> [f64; 3], tol: f64, room: usize, kind: Curved) -> Fit {
    if !(tol.is_finite() && tol > 0.0) {
        return KEPT;
    }
    let (pos, uvs) = (poly.positions(), poly.uv_coords());
    let mut verts: Vec<Vert> = Vec::new();
    let mut ids: HashMap<(usize, usize), u32> = HashMap::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for face in poly.face_iter() {
        if face.len() != 3 {
            return KEPT;
        }
        let mut t = [0u32; 3];
        for (k, c) in face.iter().enumerate() {
            let Some(ui) = c.uv else { return KEPT };
            let id = match ids.get(&(c.pos, ui)) {
                Some(id) => *id,
                None => {
                    let (Some(p), Some(uv)) = (pos.get(c.pos), uvs.get(ui)) else { return KEPT };
                    let v = Vert { uv: [uv.x, uv.y], p: [p.x, p.y, p.z] };
                    // The mesh's parameters must give the mesh's points, or
                    // they are not the plane it was triangulated in.
                    let Ok(id) = u32::try_from(verts.len()) else { return KEPT };
                    let apart = dist(at(v.uv[0], v.uv[1]), v.p);
                    if apart.is_nan() || apart > tol * 1e-3 {
                        return KEPT;
                    }
                    verts.push(v);
                    ids.insert((c.pos, ui), id);
                    id
                }
            };
            t[k] = id;
        }
        tris.push(t);
    }
    // An inside edge is split until its facets' bound meets the pass
    // tolerance, so what is reported is what was asked for (#2718). The
    // limit was twice the tolerance, reported as the tolerance.
    let limit = tol / kind.inside();
    let mut changed = false;
    // `left`: the highest inside edge over the limit; `high`: the highest
    // edge of all, both of the facets the round measured.
    let (mut left, mut high) = (0.0f64, 0.0f64);
    for round in 0..=ROUNDS {
        let mut uses: HashMap<(u32, u32), u8> = HashMap::with_capacity(tris.len() * 2);
        for t in &tris {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                let n = uses.entry(key(a, b)).or_insert(0);
                *n = n.saturating_add(1);
            }
        }
        // In edge order, so the facets are the same on every run.
        let mut edges: Vec<((u32, u32), u8)> = uses.into_iter().collect();
        edges.sort_unstable();
        (left, high) = (0.0, 0.0);
        let mut mids: HashMap<(u32, u32), u32> = HashMap::new();
        for ((a, b), n) in edges {
            let (Some(va), Some(vb)) = (verts.get(a as usize), verts.get(b as usize)) else { return KEPT };
            let uv = [0.5 * (va.uv[0] + vb.uv[0]), 0.5 * (va.uv[1] + vb.uv[1])];
            let chord = [0.5 * (va.p[0] + vb.p[0]), 0.5 * (va.p[1] + vb.p[1]), 0.5 * (va.p[2] + vb.p[2])];
            let p = at(uv[0], uv[1]);
            // A height that cannot be read is an unbounded one.
            let d = Some(dist(p, chord)).filter(|d| d.is_finite()).unwrap_or(f64::INFINITY);
            high = high.max(d);
            // Only an inside edge is split; a boundary edge is measured.
            if n != 2 || d <= limit {
                continue;
            }
            left = left.max(d);
            let id = u32::try_from(verts.len()).ok().filter(|_| round < ROUNDS && p.iter().all(|c| c.is_finite()));
            if let Some(id) = id {
                verts.push(Vert { uv, p });
                mids.insert((a, b), id);
            }
        }
        if mids.is_empty() {
            break;
        }
        let mut next = Vec::with_capacity(tris.len() + 2 * mids.len());
        for t in &tris {
            split(*t, &mids, &mut next);
        }
        if next.len() > room {
            break; // `left` and `high` are of the facets kept
        }
        tris = next;
        changed = true;
    }
    // Every exit above leaves `tris` as the round measured them: a round
    // that splits is followed by one that measures its facets, and the
    // last round never splits (no id at ROUNDS).
    if changed {
        // A flip adds an edge after that measurement. Include every new
        // edge's height, while keeping the old bound conservatively.
        high = high.max(relax(&verts, &mut tris, at, limit));
    }
    let point = |i: u32| verts.get(i as usize).map(|v| P3::new(v.p[0], v.p[1], v.p[2]));
    let facets = changed
        .then(|| tris.iter().map(|t| Some(Triangle::new(point(t[0])?, point(t[1])?, point(t[2])?))).collect::<Option<Vec<_>>>())
        .flatten();
    Fit { facets, off: high * kind.inside(), left }
}

/// The angle at `c` of the facet `a b c`, in space. `None` for a facet with
/// a null side.
fn angle_at(verts: &[Vert], c: u32, a: u32, b: u32) -> Option<f64> {
    let (pc, pa, pb) = (verts.get(c as usize)?.p, verts.get(a as usize)?.p, verts.get(b as usize)?.p);
    let (u, w) = ([pa[0] - pc[0], pa[1] - pc[1], pa[2] - pc[2]], [pb[0] - pc[0], pb[1] - pc[1], pb[2] - pc[2]]);
    let m = ((u[0] * u[0] + u[1] * u[1] + u[2] * u[2]) * (w[0] * w[0] + w[1] * w[1] + w[2] * w[2])).sqrt();
    let cos = (u[0] * w[0] + u[1] * w[1] + u[2] * w[2]) / m;
    (m > 0.0 && cos.is_finite()).then(|| cos.clamp(-1.0, 1.0).acos())
}

/// Twice the signed area of `a b c` in the parameter plane.
fn uv_area2(verts: &[Vert], [a, b, c]: [u32; 3]) -> Option<f64> {
    let (a, b, c) = (verts.get(a as usize)?.uv, verts.get(b as usize)?.uv, verts.get(c as usize)?.uv);
    Some((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]))
}

/// How many facets have an edge, and two of their (facet, opposite corner).
type Sides = (u8, [(usize, u32); 2]);

/// Flip the inside edges of a refined face that leave a flat "cap" facet
/// (#2713). The split measures edges only: three points on the surface with
/// the apex over the long edge's middle pass it, but the facet's normal
/// stands far off the surface's, and the pick shows its edges as creases
/// on a smooth wall. The rule is Lawson's ("Software for C1 surface
/// interpolation", 1977): an edge whose two opposite angles sum to more
/// than a half turn is replaced by the other diagonal. A flip is made only
/// where the two new facets keep their winding in the parameter plane and
/// the new edge passes the chord-height `limit` of the split, so the #2600
/// law holds. No point moves and no boundary edge changes. The return
/// value bounds the chord height of every added edge.
fn relax(verts: &[Vert], tris: &mut [[u32; 3]], at: &dyn Fn(f64, f64) -> [f64; 3], limit: f64) -> f64 {
    let mut high = 0.0f64;
    let mut sides: HashMap<(u32, u32), Sides> = HashMap::with_capacity(tris.len() * 2);
    for (i, t) in tris.iter().enumerate() {
        for (a, b, c) in [(t[0], t[1], t[2]), (t[1], t[2], t[0]), (t[2], t[0], t[1])] {
            let s = sides.entry(key(a, b)).or_insert((0, [(0, 0); 2]));
            if let Some(slot) = s.1.get_mut(s.0 as usize) {
                *slot = (i, c);
            }
            s.0 = s.0.saturating_add(1);
        }
    }
    // In edge order, so the facets are the same on every run.
    let mut queue: Vec<(u32, u32)> = sides.iter().filter(|(_, s)| s.0 == 2).map(|(e, _)| *e).collect();
    queue.sort_unstable_by(|x, y| y.cmp(x));
    let mut left = tris.len().saturating_mul(FLIPS_PER_FACET);
    while let Some(e) = queue.pop() {
        let Some(&(2, [(i, c), (j, d)])) = sides.get(&e) else { continue };
        let (Some(ti), Some(tj)) = (tris.get(i).copied(), tris.get(j).copied()) else { continue };
        if i == j || c == d || sides.contains_key(&key(c, d)) {
            continue;
        }
        let (Some(at_c), Some(at_d)) = (angle_at(verts, c, e.0, e.1), angle_at(verts, d, e.0, e.1)) else { continue };
        if at_c + at_d <= std::f64::consts::PI + FLIP_MARGIN {
            continue;
        }
        // Facet `i` runs a -> b -> c, and `j` runs b -> a -> d.
        let (a, b) = if ti.iter().zip(ti.iter().cycle().skip(1)).any(|(x, y)| (*x, *y) == e) { e } else { (e.1, e.0) };
        let (n1, n2) = ([a, d, c], [d, b, c]);
        let (Some(w0), Some(w1), Some(w2)) = (uv_area2(verts, ti), uv_area2(verts, n1), uv_area2(verts, n2)) else { continue };
        if !(w0 != 0.0 && w1 * w0 > 0.0 && w2 * w0 > 0.0) || uv_area2(verts, tj).is_none_or(|w| w * w0 <= 0.0) {
            continue;
        }
        let (Some(vc), Some(vd)) = (verts.get(c as usize), verts.get(d as usize)) else { continue };
        let chord = [0.5 * (vc.p[0] + vd.p[0]), 0.5 * (vc.p[1] + vd.p[1]), 0.5 * (vc.p[2] + vd.p[2])];
        let off = dist(at(0.5 * (vc.uv[0] + vd.uv[0]), 0.5 * (vc.uv[1] + vd.uv[1])), chord);
        if off.is_nan() || off > limit {
            continue;
        }
        let Some(left_now) = left.checked_sub(1) else { break };
        left = left_now;
        high = high.max(off);
        if let Some(t) = tris.get_mut(i) {
            *t = n1;
        }
        if let Some(t) = tris.get_mut(j) {
            *t = n2;
        }
        sides.remove(&e);
        sides.insert(key(c, d), (2, [(i, a), (j, b)]));
        // The four outer edges: the facet on this side, and its far corner.
        for (x, y, was, now) in [(a, d, j, (i, c)), (d, b, j, (j, c)), (b, c, i, (j, d)), (c, a, i, (i, d))] {
            if let Some(s) = sides.get_mut(&key(x, y)) {
                let n = usize::from(s.0.min(2));
                if let Some(slot) = s.1.iter_mut().take(n).find(|slot| slot.0 == was) {
                    *slot = now;
                }
            }
            queue.push(key(x, y));
        }
    }
    high
}

/// One facet split by its marked edges, winding kept: one mark halves it,
/// two cut the corner between them off and halve the rest, three make the
/// four similar facets.
fn split([a, b, c]: [u32; 3], mids: &HashMap<(u32, u32), u32>, out: &mut Vec<[u32; 3]>) {
    let m = |x: u32, y: u32| mids.get(&key(x, y)).copied();
    match (m(a, b), m(b, c), m(c, a)) {
        (None, None, None) => out.push([a, b, c]),
        (Some(ab), None, None) => out.extend([[a, ab, c], [ab, b, c]]),
        (None, Some(bc), None) => out.extend([[b, bc, a], [bc, c, a]]),
        (None, None, Some(ca)) => out.extend([[c, ca, b], [ca, a, b]]),
        (Some(ab), Some(bc), None) => out.extend([[ab, b, bc], [a, ab, bc], [a, bc, c]]),
        (None, Some(bc), Some(ca)) => out.extend([[bc, c, ca], [b, bc, ca], [b, ca, a]]),
        (Some(ab), None, Some(ca)) => out.extend([[ca, a, ab], [c, ca, ab], [c, ab, b]]),
        (Some(ab), Some(bc), Some(ca)) => out.extend([[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]]),
    }
}

#[cfg(test)]
#[path = "step_fit_tests.rs"]
mod tests;
