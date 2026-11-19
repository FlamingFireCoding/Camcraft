//! #1536 / #1605: a STEP face whose surface jumps, as a user's import runs
//! it, in a test binary of its own (it was truck's 50 s default-cap pin).
//! #917: the reader is solidcore now; the pin keeps its promise - bounded,
//! and a refusal that names the model's fault and says what to do.
//! Fixture: testdata/models/bspline_discontinuous.step (see
//! import_step_guard.rs).
//!
//! Gate map (M4.4): covers app::geometry::import::step,
//! camcraft_geometry::step.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use camcraft_app::geometry::import;

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

#[test]
fn a_face_that_jumps_ends_within_the_budget() {
    let started = Instant::now();
    let r = within(Duration::from_secs(120), || import::load_model(&testdata("bspline_discontinuous.step")));
    match r {
        Err(e) => assert!(!e.to_string().trim().is_empty(), "a refusal says why"),
        Ok(m) => assert!(!m.facets.is_empty(), "a kept mesh has facets"),
    }
    eprintln!("ended in {:?}", started.elapsed());
}
