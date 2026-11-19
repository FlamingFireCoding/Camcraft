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
//! over [`SLACK`] times the pass tolerance is split at the surface. The
//! split is by the edge alone, so the two facets that share it split alike
//! and the face stays one conforming mesh. An edge only one facet has is on
//! the face's boundary, which the neighbouring face meshes too: it is never
//! split, so the shell stays closed. What cannot be refined (the facet
//! budget, the round cap) is REPORTED as the mesh's chord error, never
//! dropped: the model then carries reduced fidelity (#864) and the gate
//! refuses to verify against it.
//!
//! Limits, stated: B-spline, NURBS and swept faces are not measured here
//! (their grid and parameter stretch are #1602's); a face whose mesh does
//! not carry usable parameters is left as the kernel made it.

use std::collections::HashMap;

use truck_meshalgo::prelude::*;
use truck_stepio::r#in::alias::{ElementarySurface, ParametricSurface, Surface};

use super::step_guard::Metered;
use crate::geometry::{Point3 as P3, Triangle};

/// An inside edge may stand this many pass tolerances off the surface.
/// The kernel accepts a grid cell on one interior point, so a mesh that
/// meets its test can stand a little past the tolerance at an edge; twice
/// the 0.005 mm cap is the program verdict's own tolerance.
pub(super) const SLACK: f64 = 2.0;
/// Refinement rounds. Each halves an edge's parameter span, so its chord
/// height falls to a quarter: 24 rounds take any span to round-off.
const ROUNDS: usize = 24;

/// One face's verdict: the facets that replace the kernel's (`None`: keep
/// them), and the chord height still over the limit (0 when none is).
pub(super) struct Fit {
    pub facets: Option<Vec<Triangle>>,
    pub off: f64,
}

const KEPT: Fit = Fit { facets: None, off: 0.0 };

/// [`fit`] for face `surface` of a shell, when its kind is measured here.
pub(super) fn fit_face(surface: Option<&Metered>, poly: &PolygonMesh, tol: f64, room: usize) -> Fit {
    let Some(s) = surface else { return KEPT };
    match &s.inner {
        Surface::ElementarySurface(e) if !matches!(**e, ElementarySurface::Plane(_)) => {}
        _ => return KEPT,
    }
    // Not through the meter: this is a measure of the finished pass.
    let at = |u: f64, v: f64| {
        let (u, v) = s.own(u, v);
        let p = s.inner.subs(u, v);
        [p.x, p.y, p.z]
    };
    let fit = fit(poly, &at, tol, room);
    if fit.off > 0.0 {
        tracing::warn!("step: a curved face's facets stand {:e} off its surface at tol {tol:e} and could not be refined", fit.off);
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

/// Hold `poly`'s inside edges within [`SLACK`] x `tol` of the surface `at`
/// evaluates in the mesh's own parameters, making at most `room` facets.
pub(super) fn fit(poly: &PolygonMesh, at: &dyn Fn(f64, f64) -> [f64; 3], tol: f64, room: usize) -> Fit {
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
                    if !(dist(at(v.uv[0], v.uv[1]), v.p) <= tol * 1e-3) {
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
    let limit = SLACK * tol;
    let mut changed = false;
    let mut off = 0.0f64;
    for round in 0..=ROUNDS {
        let mut uses: HashMap<(u32, u32), u8> = HashMap::with_capacity(tris.len() * 2);
        for t in &tris {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                let n = uses.entry(key(a, b)).or_insert(0);
                *n = n.saturating_add(1);
            }
        }
        // In edge order, so the facets are the same on every run.
        let mut inside: Vec<(u32, u32)> = uses.into_iter().filter(|&(_, n)| n == 2).map(|(e, _)| e).collect();
        inside.sort_unstable();
        off = 0.0;
        let mut mids: HashMap<(u32, u32), u32> = HashMap::new();
        for (a, b) in inside {
            let (Some(va), Some(vb)) = (verts.get(a as usize), verts.get(b as usize)) else { return KEPT };
            let uv = [0.5 * (va.uv[0] + vb.uv[0]), 0.5 * (va.uv[1] + vb.uv[1])];
            let chord = [0.5 * (va.p[0] + vb.p[0]), 0.5 * (va.p[1] + vb.p[1]), 0.5 * (va.p[2] + vb.p[2])];
            let p = at(uv[0], uv[1]);
            let d = dist(p, chord);
            if d <= limit {
                continue;
            }
            // A height that cannot be read is an unbounded one.
            off = off.max(if d.is_finite() { d } else { f64::INFINITY });
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
            break; // `off` is what the kept facets still stand off
        }
        tris = next;
        changed = true;
        off = 0.0;
    }
    let point = |i: u32| verts.get(i as usize).map(|v| P3::new(v.p[0], v.p[1], v.p[2]));
    let facets = changed
        .then(|| tris.iter().map(|t| Some(Triangle::new(point(t[0])?, point(t[1])?, point(t[2])?))).collect::<Option<Vec<_>>>())
        .flatten();
    Fit { facets, off }
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
