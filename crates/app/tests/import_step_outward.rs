//! #1617: a closed, gap-free STEP solid whose faces are all wound inward
//! (consistently, so no face disagrees with a neighbour) imports with a
//! positive signed volume; what the import notes reaches `Graded::notes`
//! and the MCP `import_model` result. A lone curved face (an open shell)
//! is a sheet: refused since #917.
//!
//! Gate map (#1617): covers app::geometry::import (parse_model_graded),
//! camcraft_geometry::step, app::commands::import_model (notes), app::script::api
//! (v_import notes) through app::mcp (import_model).
//!
//! Provenance: our own box and dome fixtures with their senses reversed
//! by hand; signed volume by the divergence theorem. No vendor material.

use std::path::{Path, PathBuf};

use camcraft_app::geometry::import::{Format, parse_model_graded};
use camcraft_app::geometry::Triangle;
use camcraft_app::mcp::Server;
use serde_json::{Value, json};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/models").join(name)
}

fn read(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("fixture")
}

/// Every FACE_SURFACE and FACE_BOUND sense of `text` reversed.
fn reversed(text: &str) -> String {
    text.lines()
        .map(|l| {
            if l.contains("FACE_SURFACE(") || l.contains("FACE_BOUND(") {
                l.replace(".T.)", ".X.)").replace(".F.)", ".T.)").replace(".X.)", ".F.)")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn volume(t: &[Triangle]) -> f64 {
    t.iter().map(|t| t.a.dot(t.b.cross(t.c)) / 6.0).sum()
}

/// #917: solidcore builds each solid outward (positive volume) itself, so
/// an inward box imports outward; truck's "turned outward" note is gone
/// with truck.
#[test]
fn an_inward_closed_box_imports_outward() {
    let plain = parse_model_graded(read("box.step").as_bytes(), Format::Step).expect("plain box");
    let v_plain = volume(&plain.mesh.facets);
    assert!(v_plain > 0.0, "the fixture is outward: {v_plain}");

    let inward = reversed(&read("box.step"));
    assert_ne!(inward, read("box.step"), "the senses were reversed");
    let got = parse_model_graded(inward.as_bytes(), Format::Step).expect("inward box imports");
    let v = volume(&got.mesh.facets);
    assert!((v - v_plain).abs() <= 1e-6 * v_plain.abs(), "imported outward: {v} vs {v_plain}");
}

/// The dome's B-spline face alone, in an open shell, wound either way: a
/// sheet, not a solid. #917: solidcore refuses it by name either way
/// (truck imported it as written).
#[test]
fn a_lone_curved_face_is_refused_as_a_sheet() {
    let text = read("bspline_dome.step");
    let shell = "#17 = CLOSED_SHELL('', (#18, #25, #32, #39, #46, #53));";
    assert_eq!(text.matches(shell).count(), 1, "fixture layout changed");
    let brep = "#16 = MANIFOLD_SOLID_BREP('', #17);";
    assert_eq!(text.matches(brep).count(), 1, "fixture layout changed");
    let lone = text.replace(shell, "#17 = OPEN_SHELL('', (#53));").replace(brep, "#16 = SHELL_BASED_SURFACE_MODEL('', (#17));");
    for (label, data) in [("as written", lone.clone()), ("reversed", reversed(&lone))] {
        let err = parse_model_graded(data.as_bytes(), Format::Step).expect_err("a sheet is not a solid").to_string();
        assert!(err.contains("not a closed solid"), "{label}: {err}");
    }
}

fn call(server: &mut Server, name: &str, args: Value) -> Value {
    let msg = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": name, "arguments": args } });
    let reply = server.handle(&msg).expect("a request gets a response");
    assert_ne!(reply["result"]["isError"], json!(true), "{name} refused: {reply}");
    reply["result"]["structuredContent"].clone()
}

/// The MCP answer carries what the import noted, line for line, and a
/// model with nothing noted says nothing.
#[test]
fn the_mcp_import_result_carries_the_notes() {
    let dir = std::env::temp_dir().join(format!("camcraft-import-outward-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let (bad, good) = (dir.join("inward.step"), dir.join("plain.step"));
    let inward = reversed(&read("box.step"));
    std::fs::write(&bad, &inward).expect("write");
    std::fs::copy(fixture("box.step"), &good).expect("copy");

    let mut server = Server::new();
    for (path, text) in [(&bad, inward), (&good, read("box.step"))] {
        let graded = parse_model_graded(text.as_bytes(), Format::Step).expect("imports");
        let got = call(&mut server, "import_model", json!({ "path": path.display().to_string() }));
        let notes: Vec<String> =
            got["notes"].as_array().map(|a| a.iter().filter_map(|n| n.as_str().map(str::to_owned)).collect()).unwrap_or_default();
        assert_eq!(notes, graded.notes, "{got}");
        if graded.notes.is_empty() {
            assert!(got.get("notes").is_none(), "nothing noted, nothing said: {got}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
