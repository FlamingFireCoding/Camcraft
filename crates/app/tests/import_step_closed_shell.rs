//! #1573: a STEP shell the file declares closed must mesh closed, or the
//! import is refused by name. Never a model with a wall missing.
//!
//! Fixture: `testdata/models/fillet_corner.step` with its corner blend's
//! weights bent (`1E300, 1E-300`). truck returned no polygon for that blend
//! (#1573). #917: the reader is solidcore now; its verdict on the bent
//! blend is its own, and the law these pins hold is the same: a refusal
//! that says why, or a mesh with no open edge.
//!
//! `box.step` with one face's `same_sense` flipped is a face wound against
//! its neighbours: closed geometry, imported outward, never called open
//! (#1573 review: two healthy CAD exports were refused that way).
//!
//! Gate map (M4.4): covers app::geometry::import::step,
//! camcraft_geometry::step, app::commands::import_model (notes),
//! app::script::api (v_import notes).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use camcraft_app::geometry::import::{self, Format, ImportError};
use camcraft_app::mcp::Server;
use serde_json::{json, Value};

fn model(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

fn within<R: Send + 'static>(limit: Duration, f: impl FnOnce() -> R + Send + 'static) -> R {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(limit) {
        Ok(r) => r,
        Err(_) => panic!("import did not finish within {limit:?}"),
    }
}

/// The fixture with its corner blend's weights bent.
fn bent() -> String {
    let text = std::fs::read_to_string(model("fillet_corner.step")).expect("fixture");
    let weights = "RATIONAL_B_SPLINE_SURFACE(((1.0,1.0,2.0),";
    assert_eq!(text.matches(weights).count(), 1, "fixture layout changed");
    assert!(text.contains("#1288=CLOSED_SHELL('',("), "fixture layout changed");
    text.replace(weights, "RATIONAL_B_SPLINE_SURFACE(((1.0E300,1.0E-300,2.0),")
}

fn import(text: String) -> Result<camcraft_app::geometry::Mesh, ImportError> {
    within(Duration::from_secs(90), move || import::parse_model(text.as_bytes(), Format::Step))
}

/// The edges of `m` that only one facet uses (vertices matched to 1e-6
/// mm): zero for a mesh with no wall missing.
fn open_edges(m: &camcraft_app::geometry::Mesh) -> usize {
    let key = |p: camcraft_app::geometry::Point3| ((p.x * 1e6).round() as i64, (p.y * 1e6).round() as i64, (p.z * 1e6).round() as i64);
    let mut uses: HashMap<_, i32> = HashMap::new();
    for t in m.facets.iter() {
        let (a, b, c) = (key(t.a), key(t.b), key(t.c));
        for (u, v) in [(a, b), (b, c), (c, a)] {
            if u != v {
                *uses.entry(if u < v { (u, v) } else { (v, u) }).or_default() += 1;
            }
        }
    }
    uses.values().filter(|&&n| n == 1).count()
}

/// A refusal that says why, or a mesh with no open edge.
fn closed_or_refused(r: Result<camcraft_app::geometry::Mesh, ImportError>) {
    match r {
        Err(e) => assert!(!e.to_string().trim().is_empty(), "a refusal says why"),
        Ok(m) => assert_eq!(open_edges(&m), 0, "a closed shell is never handed out open"),
    }
}

#[test]
fn a_closed_shell_with_a_bent_blend_is_closed_or_refused() {
    closed_or_refused(import(bent()));
}

#[test]
fn a_bent_blend_in_an_open_shell_is_closed_or_refused() {
    // Declared open, and no solid naming it: solidcore reads solids only,
    // so a sheet body is refused; it never imports with a wall missing.
    let text = bent()
        .replace("#1288=CLOSED_SHELL(", "#1288=OPEN_SHELL(")
        .replace("#1289=MANIFOLD_SOLID_BREP('',#1288);", "#1289=SHELL_BASED_SURFACE_MODEL('',(#1288));");
    closed_or_refused(import(text));
}

#[test]
fn the_outer_shell_of_a_solid_is_closed_whatever_its_keyword() {
    // A MANIFOLD_SOLID_BREP's outer shell is closed by the schema, even
    // when the file spells it OPEN_SHELL.
    let text = std::fs::read_to_string(model("fillet_corner.step"))
        .expect("fixture")
        .replace("#1288=CLOSED_SHELL(", "#1288=OPEN_SHELL(");
    assert!(text.contains("#1289=MANIFOLD_SOLID_BREP('',#1288);"), "fixture layout changed");
    closed_or_refused(import(text));
}

/// Signed volume of a mesh (divergence theorem), as tetrahedra to a point
/// off every face's plane: positive when it is wound outward, and every
/// face counts, so one face wound inward changes it.
fn volume(m: &camcraft_app::geometry::Mesh) -> f64 {
    let o = camcraft_app::geometry::Point3::new(-3.0, -7.0, -11.0);
    m.facets.iter().map(|t| (t.a - o).dot((t.b - o).cross(t.c - o)) / 6.0).sum()
}

/// `box.step` with face #32 wound against its neighbours.
fn reversed_box() -> String {
    let text = std::fs::read_to_string(model("box.step")).expect("fixture");
    let face = "#32 = FACE_SURFACE('', (#33), #90, .T.);";
    assert_eq!(text.matches(face).count(), 1, "fixture layout changed");
    let bound = "#33 = FACE_BOUND('', #34, .T.);";
    assert_eq!(text.matches(bound).count(), 1, "fixture layout changed");
    text.replace(face, "#32 = FACE_SURFACE('', (#33), #90, .F.);").replace(bound, "#33 = FACE_BOUND('', #34, .F.);")
}

#[test]
fn a_face_wound_against_its_neighbours_imports_flipped_not_refused() {
    let good = import(std::fs::read_to_string(model("box.step")).expect("fixture")).expect("box.step imports");
    let mesh = import(reversed_box()).expect("a reversed face is closed geometry: imported, not refused");
    assert_eq!(open_edges(&mesh), 0, "imported closed");
    let (v, w) = (volume(&mesh), volume(&good));
    assert!(w > 0.0, "box.step is wound outward: {w}");
    assert!((v - w).abs() <= w * 1e-6, "the reversed face is meshed outward: {v} vs {w}");
}

#[test]
fn every_solid_step_fixture_still_imports() {
    for name in [
        "box.step",
        "bspline_dome.step",
        "fillet_corner.step",
        "Milling-test.step",
        "Milling-test-inch.step",
        "sample_ap214.step",
        "lathe_holder_offset.step",
        "tool_csys_box_iso13399.step",
    ] {
        let path = model(name);
        let r = within(Duration::from_secs(240), move || import::load_model(&path));
        assert!(r.is_ok(), "{name}: {:?}", r.err());
    }
}

/// One MCP `tools/call`, in process: the structured answer, or the refusal.
fn mcp_import(path: &Path) -> Value {
    let mut server = Server::new();
    let init = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } });
    server.handle(&init).expect("initialize answers");
    let args = json!({ "path": path.display().to_string(), "name": "box" });
    let call = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": { "name": "import_model", "arguments": args } });
    let reply = server.handle(&call).expect("tools/call answers");
    assert_ne!(reply["result"]["isError"], json!(true), "import_model refused: {reply}");
    reply["result"]["structuredContent"].clone()
}

#[test]
fn the_import_notes_reach_the_mcp_answer() {
    // #1576 review: what the reader notes is said where the user and an
    // agent read the import, not only in the log. #917: truck's outward
    // flip note is gone (solidcore meshes each face outward itself); the
    // pin holds the path: the MCP answer carries the import's notes, and a
    // model with nothing to say says nothing.
    let bad = reversed_box();
    let graded = within(Duration::from_secs(90), move || import::parse_model_graded(bad.as_bytes(), Format::Step))
        .expect("a reversed face imports");
    let dir = std::env::temp_dir().join(format!("camcraft-closed-shell-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let (bad_path, good_path) = (dir.join("reversed.step"), dir.join("good.step"));
    std::fs::write(&bad_path, reversed_box()).expect("write");
    std::fs::copy(model("box.step"), &good_path).expect("copy");
    let said = mcp_import(&bad_path);
    let notes: Vec<String> = said["notes"].as_array().map(|a| a.iter().filter_map(|n| n.as_str().map(str::to_owned)).collect()).unwrap_or_default();
    assert_eq!(notes, graded.notes, "{said}");
    let good = import::load_model_graded(&good_path).expect("box.step imports");
    let quiet = mcp_import(&good_path);
    if good.notes.is_empty() {
        assert!(quiet.get("notes").is_none(), "nothing noted, nothing said: {quiet}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
