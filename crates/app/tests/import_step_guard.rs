//! #1536: a STEP face whose surface jumps is refused by name inside a
//! bounded time, never a hang; smooth B-spline faces still import;
//! degenerate B-spline input still ends cleanly. #917: the reader is
//! solidcore now; truck's per-face evaluation cap and its finer-pass
//! fallback (#1568, #1599) are gone with truck, and their pins became the
//! solidcore counterparts below (chord cap, coarser chord under a small
//! budget, a hostile shell refused by name).
//!
//! Fixtures (testdata/models/): `bspline_discontinuous.step` is box.step
//! with its top face a degree-1 B-spline whose interior knot is repeated
//! past its degree, so the surface jumps at u = 0.5; `bspline_dome.step` is
//! box.step with a smooth biquadratic dome on top.
//!
//! Gate map (M4.4): covers app::geometry::import::step,
//! camcraft_geometry::step.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use camcraft_app::geometry::import::{self, step};
use camcraft_app::geometry::{Mesh, Point3};

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

/// Run `f` on its own thread and fail the test - instead of hanging the
/// suite - when it has not answered within `limit`.
fn within<R: Send + 'static>(limit: Duration, f: impl FnOnce() -> R + Send + 'static) -> R {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(limit) {
        Ok(r) => r,
        Err(_) => panic!("import did not finish within {limit:?} - the hang is back"),
    }
}

/// The outer hang barrier. It is never the bound of a passing run: the
/// kernel's own checks are.
const HANG_BARRIER: Duration = Duration::from_secs(120);

fn volume(m: &Mesh) -> f64 {
    m.facets
        .iter()
        .map(|t| {
            let (a, b, c) = (t.a, t.b, t.c);
            (a.x * (b.y * c.z - b.z * c.y) - a.y * (b.x * c.z - b.z * c.x) + a.z * (b.x * c.y - b.y * c.x)) / 6.0
        })
        .sum()
}

/// A mesh handed out has only finite points.
fn all_finite(m: &Mesh) -> bool {
    m.facets.iter().all(|t| [t.a, t.b, t.c].iter().all(|p: &Point3| p.is_finite()))
}

#[test]
fn a_surface_that_jumps_ends_bounded_with_a_named_verdict() {
    let r = within(HANG_BARRIER, || import::load_model(&testdata("bspline_discontinuous.step")));
    match r {
        Err(e) => assert!(!e.to_string().trim().is_empty(), "a refusal says why"),
        Ok(m) => assert!(all_finite(&m) && !m.facets.is_empty(), "a kept mesh is finite"),
    }
}

#[test]
fn a_smooth_bspline_face_still_imports_closed_at_its_volume() {
    let m = within(Duration::from_secs(60), || import::load_model(&testdata("bspline_dome.step")))
        .expect("a smooth B-spline face imports");
    let b = m.bounds().expect("non-empty");
    // The dome's crown: 1 + 0.2 * (2 * 0.5 * 0.5)^2 = 1.05 at u = v = 0.5.
    assert!((b.max.z - 1.05).abs() < 5e-3, "dome crown at {}", b.max.z);
    let want = 1.0 + 0.2 / 9.0;
    let got = volume(&m);
    assert!((got - want).abs() < 2e-3, "volume {got}, want {want}");
    assert!(b.min.x.abs() < 1e-9 && (b.max.x - 1.0).abs() < 1e-9, "{b:?}");
}

#[test]
fn degenerate_bspline_variants_end_cleanly_and_bounded() {
    // Knots, multiplicities and points bent into nonsense: each must come
    // back - an import or a refusal - never a hang or a crash.
    let text = std::fs::read_to_string(testdata("bspline_discontinuous.step")).expect("fixture");
    let knots = "(2, 2, 2), (2, 2), (0.0, 0.5, 1.0), (0.0, 1.0)";
    assert!(text.contains(knots), "fixture layout changed");
    let variants = [
        "(2, 2, 2), (2, 2), (0.0, 0.0, 1.0), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (0.0, 0.5, 0.5), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (1.0, 0.5, 0.0), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (0.0, 0.5, 1.0), (1.0, 1.0)",
        "(3, 3, 3), (2, 2), (0.0, 0.5, 1.0), (0.0, 1.0)",
        "(1, 1, 1), (2, 2), (0.0, 0.5, 1.0), (0.0, 1.0)",
        "(2, 2, 2), (2, 2), (0.0, 0.5, 1.0E300), (0.0, 1.0)",
    ];
    for v in variants {
        let data = text.replace(knots, v).into_bytes();
        let started = Instant::now();
        let r = within(Duration::from_secs(60), move || import::parse_model(&data, import::Format::Step));
        if let Ok(m) = &r {
            assert!(all_finite(m), "{v}");
        }
        eprintln!("{v}: {} in {:?}", r.map(|m| format!("{} facets", m.len())).unwrap_or_else(|e| e.to_string()), started.elapsed());
    }
}

/// #864 / #917: a model that fits the facet budget is meshed at the
/// importer's chord cap, never coarser.
#[test]
fn the_dome_meshes_at_the_chord_cap() {
    let data = std::fs::read(testdata("bspline_dome.step")).expect("fixture");
    let (m, chord) = within(Duration::from_secs(60), move || step::parse_with_chord(&data)).expect("the dome imports");
    assert!((chord - step::CHORD_CAP).abs() < 1e-12, "chord {chord}, cap {}", step::CHORD_CAP);
    assert!(step::coarse(chord).is_none(), "full fidelity");
    let want = 1.0 + 0.2 / 9.0;
    assert!((volume(&m) - want).abs() < 2e-3, "volume {}, want {want}", volume(&m));
}

/// #1054 / #917: a model over a small facet budget is meshed again at a
/// coarser chord. The kept mesh reports that chord, has fewer facets and
/// keeps its volume.
#[test]
fn a_small_budget_keeps_a_coarser_mesh_and_reports_its_chord() {
    let data = std::fs::read(testdata("bspline_dome.step")).expect("fixture");
    let (full, full_chord) = step::parse_with_chord(&data).expect("the dome imports");
    let budget = full.len() / 3;
    let (m, chord) = within(Duration::from_secs(60), move || step::parse_with_chord_budget(&data, budget))
        .expect("a smooth model over a small budget keeps a coarser mesh");
    assert!(chord > full_chord, "the kept mesh reports its own, coarser chord: {chord} vs {full_chord}");
    assert!(step::coarse(chord).is_some(), "reduced fidelity is reported");
    assert!(m.len() <= budget && m.len() < full.len(), "{} facets kept, budget {budget}, {} at full detail", m.len(), full.len());
    let want = 1.0 + 0.2 / 9.0;
    assert!((volume(&m) - want).abs() < 1e-2, "volume {}, want {want}", volume(&m));
}

/// #1599: a hostile shell (the fixture's jumping face #53 listed 40 more
/// times) ends bounded. A shell that lists one face 41 times is not a
/// closed solid: it is refused by name.
#[test]
fn a_shell_of_many_repeated_faces_is_refused_bounded() {
    let text = std::fs::read_to_string(testdata("bspline_discontinuous.step")).expect("fixture");
    let shell = "CLOSED_SHELL('', (#18, #25, #32, #39, #46, #53));";
    assert!(text.contains(shell), "the fixture's shell line");
    let hostile = text.replace(shell, &format!("CLOSED_SHELL('', (#18, #25, #32, #39, #46, #53{}));", ", #53".repeat(40)));
    let r = within(HANG_BARRIER, move || import::parse_model(hostile.as_bytes(), import::Format::Step));
    let err = r.expect_err("refused").to_string();
    assert!(!err.trim().is_empty(), "{err}");
}
