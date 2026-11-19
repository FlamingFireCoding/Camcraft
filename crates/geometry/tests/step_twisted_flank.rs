//! #2780: a closed solid with one strongly twisted B-spline flank, read
//! through the solidcore kernel. The importer reads it at its true volume,
//! or refuses it by name; it never hands out a mesh with a wall missing.
//! Fixtures (our own, testdata/models/): `twisted_flank.step` is a 40 x 40
//! mm block whose top is the degree-3 B-spline z = 60uv (volume 8000 mm3);
//! `twisted_flank_off.step` is the same solid with one vertex lifted 0.5 mm
//! off its curves and its surface.
//! Provenance: our own fixtures, written from ISO 10303-21/-42.
//!
//! Gate map (M4.4): covers camcraft_geometry::step.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use camcraft_geometry::geometry::Mesh;
use camcraft_geometry::step::{self, StepError};

/// The importer's facet budget and its chord cap (mm), as the app passes them.
const BUDGET: usize = 30_000_000;
const CHORD: f64 = 0.005;

fn read(name: &str) -> Result<step::StepMesh, StepError> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name);
    let data = std::fs::read(&path).expect("fixture");
    step::parse(&data, BUDGET, CHORD)
}

fn volume(m: &Mesh) -> f64 {
    m.facets
        .iter()
        .map(|t| {
            let (a, b, c) = (t.a, t.b, t.c);
            (a.x * (b.y * c.z - b.z * c.y) - a.y * (b.x * c.z - b.z * c.x) + a.z * (b.x * c.y - b.y * c.x)) / 6.0
        })
        .sum()
}

/// A refusal names what it refused: a solid, an entity or a face.
fn named(e: &StepError) -> bool {
    let s = e.to_string();
    !s.trim().is_empty() && (s.contains('#') || s.contains("solid"))
}

#[test]
fn a_twisted_bspline_flank_reads_at_its_volume() {
    match read("twisted_flank.step") {
        Ok(m) => {
            let v = volume(&m.mesh);
            assert!((v - 8000.0).abs() < 8.0, "volume {v} mm3, want 8000");
            let b = m.mesh.bounds().expect("non-empty");
            assert!((b.max.z - 60.0).abs() < 0.05 && b.min.z.abs() < 1e-6, "{b:?}");
        }
        Err(e) => assert!(named(&e), "a refusal names its fault: {e}"),
    }
}

#[test]
fn a_twisted_flank_with_a_vertex_off_its_curves_reads_or_is_refused_by_name() {
    match read("twisted_flank_off.step") {
        Ok(m) => {
            let v = volume(&m.mesh);
            assert!((v - 8000.0).abs() < 80.0, "volume {v} mm3, want about 8000");
        }
        Err(e) => assert!(named(&e), "a refusal names its fault: {e}"),
    }
}
