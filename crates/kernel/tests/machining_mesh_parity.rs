//! The machining mesh's parity laws for the STEP switch (#917): positions
//! snapped off their last-bit noise and never moved past the 1e-9 mm
//! window, a solid whose faces all point inward turned outward and said,
//! and the fillet fixture meshed at the app chord (adaptive refinement of
//! the deviation bound).
//! Provenance: our own test design; fixtures are the repo's own.
//!
//! Gate map (#917): covers camcraft_kernel::mesh_weld,
//! camcraft_kernel::machining_mesh, camcraft_kernel::mesh_deviation.

use std::path::PathBuf;

use camcraft_kernel::machining_mesh::machining_mesh;
use camcraft_kernel::mesh_weld::{SNAP_WINDOW_MM, snap};
use camcraft_kernel::solidcore_impl::SolidcoreKernel;
use camcraft_kernel::solidcore_step::{StepSolid, import_step};
use solidcore::kernel::CancelToken;

/// The app's chord cap (#689), mm.
const TOL: f64 = 0.005;

fn text(file: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata/models")
        .join(file);
    std::fs::read_to_string(&path).unwrap_or_default()
}

fn solid_of(file: &str, text: &str) -> StepSolid {
    let mut k = SolidcoreKernel::default();
    let mut import = match import_step(&mut k, text.as_bytes(), &CancelToken::new()) {
        Ok(i) => i,
        Err(e) => panic!("{file}: {e}"),
    };
    assert_eq!(import.solids.len(), 1, "{file}: one solid expected");
    import.solids.remove(0)
}

#[test]
fn the_snap_takes_the_last_bit_noise_off_a_grid_value() {
    for (noisy, exact) in [
        (20.000000000000004, 20.0_f64),
        (19.999999999999996, 20.0),
        (-5.000000000000001, -5.0),
        (14.999999999999998, 15.0),
        (0.1 + 0.2, 0.3),
        (-1e-17, 0.0),
        (12.5 + 1e-10, 12.5),
    ] {
        assert_eq!(snap(noisy).to_bits(), exact.to_bits(), "{noisy:e} -> {exact}");
    }
    // A grid value stays itself, and the result is +0.0, never -0.0.
    for exact in [0.0_f64, 1.0, 0.001, 123.456, -98.765] {
        assert_eq!(snap(exact).to_bits(), exact.to_bits(), "{exact}");
    }
    assert_eq!(snap(-0.0).to_bits(), 0.0_f64.to_bits());
}

#[test]
fn a_coordinate_outside_the_window_is_never_moved() {
    for c in [
        12.5 + 1e-6,
        12.5 - 1e-6,
        20.0 + 2.0 * SNAP_WINDOW_MM,
        12.499_999_9,
        std::f64::consts::PI,
        12.507_3,
        f64::NAN,
        f64::INFINITY,
        1e300,
    ] {
        assert_eq!(snap(c).to_bits(), c.to_bits(), "{c:e} must stay");
    }
}

#[test]
fn the_bore_mesh_stands_on_its_exact_levels() {
    let s = solid_of("Milling-test.step", &text("Milling-test.step"));
    let m = machining_mesh(&s, TOL, &CancelToken::new()).unwrap_or_else(|e| panic!("{e}"));
    let zs: Vec<f64> = m.positions.iter().map(|p| p[2]).collect();
    let top = zs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let bottom = zs.iter().copied().fold(f64::INFINITY, f64::min);
    assert_eq!((top, bottom), (20.0, -5.0), "no last-bit noise on the levels");
    for z in [0.0, 20.0] {
        assert!(zs.contains(&z), "a vertex stands exactly on Z {z}");
    }
    assert_eq!(m.report.open_sides, 0);
}

/// box.step with every face and bound sense turned over: the same closed
/// box, all faces pointing inward.
fn inward(text: &str) -> String {
    text.lines()
        .map(|l| {
            if l.contains("FACE_SURFACE(") || l.contains("FACE_BOUND(") {
                l.replace(".T.)", ".X.)")
                    .replace(".F.)", ".T.)")
                    .replace(".X.)", ".F.)")
            } else {
                l.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn an_inward_solid_is_turned_outward_and_said() {
    let plain = solid_of("box.step", &text("box.step"));
    let good = machining_mesh(&plain, TOL, &CancelToken::new()).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !good.report.notes.iter().any(|n| n.contains("inward")),
        "an outward box says nothing: {:?}",
        good.report.notes
    );
    let turned = inward(&text("box.step"));
    assert_ne!(turned, text("box.step"), "the senses were turned");
    let s = solid_of("box.step inward", &turned);
    let m = machining_mesh(&s, TOL, &CancelToken::new())
        .unwrap_or_else(|e| panic!("an inward closed box is the same box: {e}"));
    assert!(m.report.volume > 0.0);
    assert!((m.report.volume - good.report.volume).abs() <= good.report.volume * 1e-9);
    assert!(
        m.report.notes.iter().any(|n| n.contains("inward") && n.contains("turned outward")),
        "the turn is said: {:?}",
        m.report.notes
    );
}

#[test]
fn the_fillet_fixture_meshes_at_the_app_chord() {
    let s = solid_of("fillet_corner.step", &text("fillet_corner.step"));
    let m = machining_mesh(&s, TOL, &CancelToken::new())
        .unwrap_or_else(|e| panic!("fillet_corner.step at {TOL} mm: {e}"));
    assert!(m.report.deviation.largest() <= TOL, "{:?}", m.report.deviation);
    assert_eq!(m.report.open_sides, 0);
    assert!(m.report.volume > 0.0);
}
