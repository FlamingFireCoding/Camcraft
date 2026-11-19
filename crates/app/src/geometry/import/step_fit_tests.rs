//! Unit tests of the curved-face fit's reported height (#2718): the value
//! the fit reports for a face is never below the largest distance that the
//! kept facets stand off the true surface, for each curved kind it measures.
//! The meshes are made here on a triangular lattice in the surface's own
//! parameters, coarse enough that their edges stand between one and two
//! pass tolerances off: the band the fit keeps and did not report.

use super::*;
use truck_meshalgo::prelude::{Faces, PolygonMesh, StandardAttributes, Vector2};

const TOL: f64 = 0.005;

type At = dyn Fn(f64, f64) -> [f64; 3];
/// A facet corner: the point, its parameters, its normal.
type Corner = (usize, Option<usize>, Option<usize>);

/// A lattice of `nu` x `nv` points from `(u0, v0)`, steps `du`, `dv`, odd
/// rows shifted half a step, so the facets are near equilateral in (u, v).
fn lattice(at: &At, (u0, du, nu): (f64, f64, usize), (v0, dv, nv): (f64, f64, usize)) -> PolygonMesh {
    let mut positions = Vec::new();
    let mut uv_coords = Vec::new();
    for j in 0..nv {
        for i in 0..nu {
            let u = u0 + (i as f64 + 0.5 * (j % 2) as f64) * du;
            let v = v0 + j as f64 * dv;
            let p = at(u, v);
            positions.push(truck_meshalgo::prelude::Point3::new(p[0], p[1], p[2]));
            uv_coords.push(Vector2::new(u, v));
        }
    }
    let id = |i: usize, j: usize| j * nu + i;
    let mut tris: Vec<[usize; 3]> = Vec::new();
    for j in 0..nv - 1 {
        for i in 0..nu - 1 {
            if j % 2 == 0 {
                tris.push([id(i, j), id(i + 1, j), id(i, j + 1)]);
                tris.push([id(i + 1, j), id(i + 1, j + 1), id(i, j + 1)]);
            } else {
                tris.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
                tris.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            }
        }
    }
    let faces: Vec<[Corner; 3]> = tris.iter().map(|t| t.map(|k| (k, Some(k), None))).collect();
    let faces = Faces::from_iter(faces.iter().map(|f| &f[..]));
    PolygonMesh::new(StandardAttributes { positions, uv_coords, normals: Vec::new() }, faces)
}

/// The kept facets: the fit's own, or the kernel's when it kept them.
fn kept(fit: &Fit, poly: &PolygonMesh) -> Vec<[[f64; 3]; 3]> {
    if let Some(f) = &fit.facets {
        return f.iter().map(|t| [t.a, t.b, t.c].map(|p| [p.x, p.y, p.z])).collect();
    }
    let pos = poly.positions();
    poly.face_iter().map(|f| [0, 1, 2].map(|k| {
        let p = pos[f[k].pos];
        [p.x, p.y, p.z]
    })).collect()
}

/// The largest distance a dense barycentric sample of the facets stands
/// off the surface `gap` measures.
fn worst(facets: &[[[f64; 3]; 3]], gap: &dyn Fn([f64; 3]) -> f64) -> f64 {
    const N: usize = 12;
    let mut w = 0.0f64;
    for t in facets {
        for a in 0..=N {
            for b in 0..=N - a {
                let (la, lb) = (a as f64 / N as f64, b as f64 / N as f64);
                let lc = 1.0 - la - lb;
                let p = [0, 1, 2].map(|k| la * t[0][k] + lb * t[1][k] + lc * t[2][k]);
                w = w.max(gap(p));
            }
        }
    }
    w
}

/// The pin: what the fit reports bounds what it kept, and what it kept
/// stands past the pass tolerance (the value the import reported before).
fn pins(name: &str, kind: Curved, at: &At, gap: &dyn Fn([f64; 3]) -> f64, u: (f64, f64, usize), v: (f64, f64, usize)) {
    let poly = lattice(at, u, v);
    let fit = fit(&poly, at, TOL, 1_000_000, kind).expect("the lattice is measured");
    let facets = kept(&fit, &poly);
    let w = worst(&facets, gap);
    eprintln!("{name}: reported {:.5} mm, sampled worst {w:.5} mm, {} facets", fit.off, facets.len());
    assert!(w > TOL, "{name}: the lattice must stand past the pass tolerance to pin #2718 (worst {w})");
    assert!(fit.off >= w, "{name}: reported {} mm is below the kept facets' worst {w} mm", fit.off);
    assert!(fit.off <= 2.0 * w, "{name}: reported {} mm is far over the worst {w} mm", fit.off);
}

fn rho(p: [f64; 3]) -> f64 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

#[test]
fn a_cylinder_face_reports_no_less_than_its_facets_stand_off() {
    let r = 10.0;
    let at = move |u: f64, v: f64| [r * u.cos(), r * u.sin(), v];
    let du = 0.08;
    pins("cylinder", Curved::Cylinder, &at, &|p| (r - rho(p)).abs(), (0.0, du, 12), (0.0, r * du * 0.866, 12));
}

#[test]
fn a_cone_face_reports_no_less_than_its_facets_stand_off() {
    let (s, c) = (30f64.to_radians().sin(), 30f64.to_radians().cos());
    // `v` is the slant distance from the apex.
    let at = move |u: f64, v: f64| [v * s * u.cos(), v * s * u.sin(), v * c];
    let gap = move |p: [f64; 3]| (rho(p) * c - p[2] * s).abs();
    let du = 0.12;
    pins("cone", Curved::Cone, &at, &gap, (0.0, du, 12), (10.0, 5.0 * du * 0.866, 12));
}

#[test]
fn a_sphere_face_reports_no_less_than_its_facets_stand_off() {
    let r = 10.0;
    let at = move |u: f64, v: f64| [r * v.cos() * u.cos(), r * v.cos() * u.sin(), r * v.sin()];
    let gap = move |p: [f64; 3]| (r - (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt()).abs();
    let du = 0.0775;
    pins("sphere", Curved::Sphere, &at, &gap, (0.0, du, 12), (-0.3, du * 0.866, 12));
}

#[test]
fn a_torus_face_reports_no_less_than_its_facets_stand_off() {
    let (big, small) = (20.0, 5.0);
    let at = move |u: f64, v: f64| {
        let w = big + small * v.cos();
        [w * u.cos(), w * u.sin(), small * v.sin()]
    };
    let gap = move |p: [f64; 3]| (small - ((rho(p) - big).powi(2) + p[2] * p[2]).sqrt()).abs();
    let a = 0.63;
    // The outer side (curved two ways alike) and the inner side (a saddle).
    pins("torus outer", Curved::Torus, &at, &gap, (0.0, a / 25.0, 12), (-0.3, a * 0.866 / small, 12));
    pins("torus inner", Curved::Torus, &at, &gap, (0.0, a / 15.0, 12), (std::f64::consts::PI - 0.3, a * 0.866 / small, 12));
}

/// A flip introduces a new chord. Its height must remain in the reported
/// bound, even though the split measured the previous diagonal.
#[test]
fn a_flipped_edge_reports_its_height_and_keeps_the_chord_limit() {
    let at = |u: f64, v: f64| [10.0 * u.cos(), 10.0 * u.sin(), v];
    let verts: Vec<Vert> = [[-0.2, 0.0], [0.2, 0.0], [0.02, 0.01], [-0.02, -0.01]]
        .into_iter()
        .map(|uv| Vert {
            p: at(uv[0], uv[1]),
            uv,
        })
        .collect();
    let before = [[0, 1, 2], [1, 0, 3]];
    let mut tris = before;
    let chord = [0, 1, 2].map(|k| 0.5 * (verts[2].p[k] + verts[3].p[k]));
    let height = dist(at(0.0, 0.0), chord);
    assert!(height > 0.001);
    let reported = relax(&verts, &mut tris, &at, 1.0);
    assert_ne!(tris, before, "the fixture must flip its old diagonal");
    assert!(
        reported >= height,
        "the new edge's height is missing: {reported} < {height}"
    );
    let mut refused = before;
    assert_eq!(relax(&verts, &mut refused, &at, height * 0.5), 0.0);
    assert_eq!(
        refused, before,
        "an edge above the chord limit must not replace the diagonal"
    );
}

/// The log lines `run` writes, with its result.
fn logged(run: impl FnOnce() -> Fit) -> (Fit, String) {
    use std::sync::{Arc, Mutex};
    struct Sink(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Sink {
        fn write(&mut self, d: &[u8]) -> std::io::Result<usize> {
            self.0.lock().expect("log sink").extend_from_slice(d);
            Ok(d.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let buf = Arc::new(Mutex::new(Vec::new()));
    let sink = buf.clone();
    let sub = tracing_subscriber::fmt().with_ansi(false).with_writer(move || Sink(sink.clone())).finish();
    let fit = tracing::subscriber::with_default(sub, run);
    let text = String::from_utf8_lossy(&buf.lock().expect("log sink")).into_owned();
    (fit, text)
}

/// The fallback (#2719): a face the fit cannot measure keeps the kernel's
/// facets, the log names the face and the reason, and the import reports
/// the face unmeasured, so the model is reduced fidelity and verify refuses.
fn pins_unmeasured(name: &str, index: usize, poly: &PolygonMesh, at: &At, why: &str) {
    let (kept, log) = logged(|| judged(index, fit(poly, at, TOL, 1_000_000, Curved::Cylinder)));
    eprintln!("{name}: {}", log.trim());
    assert!(kept.facets.is_none(), "{name}: the kernel's facets are kept");
    assert_eq!(kept.off, UNMEASURED, "{name}: the face is reported unmeasured");
    assert!(log.contains(&format!("face {}", index + 1)) && log.contains(why), "{name}: the log names the face and the reason: {log}");
    // Through the import's facet pass: the kernel's facets, and the height.
    let (facets, _, off) = super::super::step_facets::facets_of(&[Some(poly.clone())], 1_000_000, |i, p, room| judged(i, fit(p, at, TOL, room, Curved::Cylinder))).expect("in budget");
    assert_eq!(facets.len(), poly.face_iter().count(), "{name}: the kernel's facets");
    assert_eq!(off, UNMEASURED, "{name}: the pass carries the unmeasured height");
    // Reduced fidelity in mm for any file unit from the micrometre to the
    // kilometre, and never the unknown-fidelity sentinel.
    for k in [1e-3, 1.0, 25.4, 1e6] {
        assert!(super::super::step::coarse(UNMEASURED * k).is_some(), "{name}: reduced fidelity at {k} mm a unit");
        assert!(!crate::model_fidelity::is_unknown(UNMEASURED * k), "{name}: not the unknown sentinel at {k} mm a unit");
    }
}

fn cylinder_at(u: f64, v: f64) -> [f64; 3] {
    [10.0 * u.cos(), 10.0 * u.sin(), v]
}

#[test]
fn a_face_without_surface_parameters_is_logged_and_reported_unmeasured() {
    let poly = lattice(&cylinder_at, (0.0, 0.08, 6), (0.0, 0.7, 6));
    // The same mesh, its corners without parameters.
    let corners: Vec<Vec<Corner>> = poly.face_iter().map(|f| f.iter().map(|c| (c.pos, None, None)).collect()).collect();
    let faces = Faces::from_iter(corners.iter().map(|f| &f[..]));
    let bare = PolygonMesh::new(StandardAttributes { positions: poly.positions().to_vec(), uv_coords: Vec::new(), normals: Vec::new() }, faces);
    pins_unmeasured("no parameters", 3, &bare, &cylinder_at, "no surface parameters");
}

#[test]
fn a_face_whose_parameters_miss_its_points_is_logged_and_reported_unmeasured() {
    let poly = lattice(&cylinder_at, (0.0, 0.08, 6), (0.0, 0.7, 6));
    // A surface 0.1 mm larger than the one the mesh was made on.
    let other = |u: f64, v: f64| [10.1 * u.cos(), 10.1 * u.sin(), v];
    pins_unmeasured("parameters miss", 0, &poly, &other, "parameters do not give its points");
}

/// #2754: an edge that bounds a face curved two ways is divided by the
/// inside bound, so its face's report (edge x bound) meets the pass; a
/// cylinder wall carries its finest rim's divisor to every rim, across a
/// chain of walls; a plane's own edges stay at 1; an index past the shell
/// is skipped, never a panic.
#[test]
fn edge_divisors_hold_two_way_faces_and_match_cylinder_rims() {
    let faces = vec![
        (Some(Curved::Cone), vec![0, 1]),
        (Some(Curved::Cylinder), vec![1, 2]),
        (Some(Curved::Cylinder), vec![2, 3]),
        (None, vec![3, 4, 99]),
        (Some(Curved::Cylinder), vec![5, 6]),
    ];
    let fine = edge_divisors(7, &faces);
    let cone = Curved::Cone.inside();
    assert_eq!(fine, vec![cone, cone, cone, cone, 1.0, 1.0, 1.0]);
    assert!(edge_divisors(0, &faces).is_empty());
}
