//! #2987: the first traverse of a 3+2 operation against the machine
//! table. When the head tilts, the clearance plane drops toward the
//! table on its low side; the table's top face over the stated travel
//! must stay under it, or the post is held by name. The table counts only
//! where the move along the tool axis can carry the tool under its top: a
//! horizontal axis (B 90) never reads it, and an axis that points down
//! toward the table gets a note. When only the work
//! tilts (a rotary that turns the part), the table stays under the work
//! and is not read. A head-tilt machine that states no travel gives a
//! note that names the table as not judged - never a silent pass.
//!
//! Provenance: our own machine files and fixtures; the shop rule "never
//! traverse through uncut stock or steel" (machine-tool handbooks). No
//! vendor material.
//!
//! Gate map (#2987): covers app::operation (first_traverse, generate),
//! app::posting, sim::verify (first_traverse).

use camcraft_app::commands::{AddTool, CreateOperation, GenerateOperation, History, LoadMachine, SetStock};
use camcraft_app::machine::parse_mch;
use camcraft_app::operation::{Height, OpParams, OpStatus, SrcRef, Tilt};
use camcraft_app::schema::Location;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::verify::Warning;
use camcraft_app::{model, posting};
use flecs_ecs::prelude::World;

/// The B 30 deg plane through the block's top centre (Z 20).
const B_DEG: f64 = 30.0;
const ORIGIN_Z: f64 = 20.0;

/// How the machine reaches the tilt.
#[derive(Clone, Copy)]
enum Rig {
    /// A and B ride the head (a head hangs below them).
    Head,
    /// A and B turn the work (no head below them).
    Work,
    /// A and B ride the head; the file states no linear travel.
    HeadNoTravel,
}

/// A small 5-axis mill: X and Y travel +-100 mm, Z travel 0..300 mm, so
/// the table top is Z 0 - the bottom of the block.
fn machine_text(rig: Rig) -> String {
    let t = |lo: f64, hi: f64| match rig {
        Rig::HeadNoTravel => String::new(),
        _ => format!(r#", "travel": {{ "min": {lo}, "max": {hi} }}"#),
    };
    let rotaries = |inner: &str| {
        format!(
            r#"{{ "id": "ra", "name": "A", "type": "rotary", "axis": [1.0, 0.0, 0.0], "travel": {{ "min": -120.0, "max": 120.0 }}, "parts": [
                 {{ "id": "rb", "name": "B", "type": "rotary", "axis": [0.0, 1.0, 0.0], "travel": {{ "min": -120.0, "max": 120.0 }}{inner} }} ] }}"#
        )
    };
    let head = r#"{ "id": "hed", "name": "head", "type": "head", "spindle": { "min_speed": 50.0, "max_speed": 15000.0 } }"#;
    let (z_parts, base_extra) = match rig {
        Rig::Work => (format!(r#", "parts": [ {head} ]"#), format!(", {}", rotaries(""))),
        _ => (format!(r#", "parts": [ {} ]"#, rotaries(&format!(r#", "parts": [ {head} ]"#))), String::new()),
    };
    format!(
        r#"{{
  "general": {{ "model": "TEST 5-AXIS", "vendor": "Camcraft", "kind": "mill" }},
  "kinematics": {{ "default": {{
    "units": {{ "length": "mm", "angle": "degrees" }},
    "parts": [
      {{ "id": "base", "name": "table", "type": "table", "parts": [
        {{ "id": "crl", "name": "X", "type": "linear"{x}, "parts": [
          {{ "id": "sld", "name": "Y", "type": "linear"{y} }} ] }},
        {{ "id": "col", "name": "Z", "type": "linear"{z}, "safe_retract": 0.0{z_parts} }}{base_extra}
      ] }}
    ] }} }},
  "controller": {{ "synced_configuration": {{ "parts": {{
    "X": {{ "coordinate": 0 }}, "Y": {{ "coordinate": 1 }}, "Z": {{ "coordinate": 2 }},
    "A": {{ "coordinate": 3 }}, "B": {{ "coordinate": 4 }}
  }} }} }},
  "tooling": {{ "number_of_tools": 24 }}
}}"#,
        x = t(-100.0, 100.0),
        y = t(-100.0, 100.0),
        z = t(0.0, 300.0),
    )
}

/// A 60 x 60 x 20 block (x -30..30, z 0..20) on `rig`, a B `b_deg` face
/// with its clearance plane at world `clear` over the tilt origin.
fn generate(rig: Rig, b_deg: f64, clear: f64) -> (World, OpStatus, Vec<Warning>, Vec<String>) {
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    let m = parse_mch(&machine_text(rig)).unwrap_or_else(|e| panic!("test machine: {e}"));
    h.execute(&world, LoadMachine::new(&world, "test-5axis.mch".into(), Some(m)).expect("machine"));
    h.execute(&world, SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Location::default()).expect("stock"));
    let t = catalog().into_iter().find(|(n, _)| *n == "EM-0400-2FL").map(|(_, t)| t).expect("catalog");
    h.execute(&world, AddTool::new(&world, "EM-0400-2FL", t).expect("tool"));
    let p = OpParams::Face {
        holes: Default::default(),
        tilt: Some(Tilt { basis_version: 1, a_deg: 0.0, b_deg, origin: [0.0, 0.0, ORIGIN_Z] }),
        blend_turns: false,
        stepover: 3.0,
        clearance_z: Height::fixed(clear),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    h.execute(&world, CreateOperation::new(&world, "Face1", "EM-0400-2FL", SrcRef::Stock, p).expect("create"));
    h.execute(&world, GenerateOperation::new(&world, "Face1").expect("generate"));
    let e = model::find_by_name(&world, "Face1").expect("op");
    let status = camcraft_app::operation::status_of(&world, &e);
    let cached = model::toolpath_of(&world, &e).expect("cached path");
    let (warns, notes) = (cached.warnings.clone(), cached.path.notes.clone());
    (world, status, warns, notes)
}

fn low(warns: &[Warning]) -> Option<(f64, f64, bool)> {
    warns.iter().find_map(|w| match *w {
        Warning::FirstTraverseLow { height_mm, top_mm, fixture, .. } => Some((height_mm, top_mm, fixture)),
        _ => None,
    })
}

/// The table's top corner (x 100, z 0) along the B 30 deg axis, over the
/// tilt origin: 100 sin 30 - 20 cos 30 = 32.68 mm. The block's own high
/// corner (x 30, z 20) stands only 30 sin 30 = 15 mm there.
fn table_top() -> f64 {
    let b = B_DEG.to_radians();
    100.0 * b.sin() - ORIGIN_Z * b.cos()
}

#[test]
fn a_head_tilt_plane_under_the_table_inside_the_travel_is_refused_by_name() {
    // Z 40: 20 mm over the origin - over the block (15), under the table (32.68).
    let (world, status, warns, _) = generate(Rig::Head, B_DEG, 40.0);
    assert_eq!(status, OpStatus::Ok);
    let (h_mm, top_mm, fixture) = low(&warns).unwrap_or_else(|| panic!("no first-traverse verdict: {warns:?}"));
    assert!((h_mm - 20.0).abs() < 1e-6, "the plane along the tool axis: {h_mm}");
    assert!((top_mm - table_top()).abs() < 1e-6, "the table top along the axis: {top_mm}, want {}", table_top());
    assert!(fixture, "the table is steel, not stock");
    let err = posting::post_setup_unwritten(&world).expect_err("the program must not post");
    for text in ["Face1", "first rapid (move 0)", &format!("raise the clearance height above Z{:.3}", table_top())] {
        assert!(err.contains(text), "{text}: {err}");
    }
}

#[test]
fn a_head_tilt_plane_over_the_table_top_face_passes() {
    // Z 60: 40 mm over the origin - over the table's top face (32.68).
    // Its slab, 1e6 mm deep, reads lower along this axis and never counts.
    let (_, status, warns, notes) = generate(Rig::Head, B_DEG, 60.0);
    assert_eq!(status, OpStatus::Ok);
    assert_eq!(low(&warns), None, "{warns:?}");
    assert!(!notes.iter().any(|n| n.contains("machine table is not judged")), "{notes:?}");
}

#[test]
fn a_work_turning_rotary_keeps_the_table_under_the_work() {
    // The same plane as the refused head-tilt case: the work tilts, the
    // table stays under it, and only the block's top (15) counts.
    let (_, status, warns, notes) = generate(Rig::Work, B_DEG, 40.0);
    assert_eq!(status, OpStatus::Ok);
    assert_eq!(low(&warns), None, "{warns:?}");
    assert!(!notes.iter().any(|n| n.contains("machine table is not judged")), "{notes:?}");
}

#[test]
fn a_head_tilt_machine_with_no_travel_names_the_table_unjudged() {
    let (_, status, warns, notes) = generate(Rig::HeadNoTravel, B_DEG, 40.0);
    assert_eq!(status, OpStatus::Ok);
    assert_eq!(low(&warns), None, "no table box, so no table verdict: {warns:?}");
    let note = notes.iter().find(|n| n.contains("machine table is not judged")).unwrap_or_else(|| panic!("no note: {notes:?}"));
    assert!(note.contains("does not state all of the X, Y and Z travel"), "{note}");
    assert!(note.contains("judged against the stock and the fixtures only"), "{note}");
}

#[test]
fn a_head_tilt_past_90_names_the_table_unjudged() {
    // B 120: the axis (sin 120, 0, cos 120) points down toward the table,
    // so a HIGHER plane along it stands LOWER in the world. The table top
    // face reads -100 sin 120 + 20 cos 60 = -76.6 at its low corner; a
    // plane at 90 or 100 mm over the origin is far past it, and the move
    // along the axis to the plane can go under the table top. The verdict
    // can only say "raise", which is wrong here, so the note names the
    // table as not judged (and the slab's underside, 5e5 mm along the
    // axis, is never read as a top).
    for over in [90.0, 100.0] {
        let (_, _, warns, notes) = generate(Rig::Head, 120.0, ORIGIN_Z + over);
        assert_eq!(low(&warns), None, "no table verdict past 90 deg: {warns:?}");
        let note = notes.iter().find(|n| n.contains("machine table is not judged")).unwrap_or_else(|| panic!("no note at {over}: {notes:?}"));
        assert!(note.contains("points down toward the table"), "{note}");
    }
}

#[test]
fn a_horizontal_head_axis_does_not_read_the_far_table_edge() {
    // B 90: the axis is world +X. The table's far edge (x 100) reads 100
    // along it, but a move along a horizontal axis keeps the world Z, so
    // the tool never goes under the table top. A plane 40 mm over the
    // origin clears the block (30 along X) and passes, with no note.
    let (_, _, warns, notes) = generate(Rig::Head, 90.0, ORIGIN_Z + 40.0);
    assert_eq!(low(&warns), None, "the table's far edge was read along a horizontal axis: {warns:?}");
    assert!(!notes.iter().any(|n| n.contains("machine table is not judged")), "{notes:?}");
}

#[test]
fn a_near_horizontal_head_axis_still_reads_the_table() {
    // B 80: the axis has a small up component (cos 80 = 0.17). The table
    // top face reads 100 sin 80 - 20 cos 80 = 95.0 along it. From a start
    // on the table top at x 100, the move along the axis to a plane 40 mm
    // over the origin drops the tool (95.0 - 40) cos 80 = 9.6 mm under
    // the table top: refused by name, with the height to clear.
    let b = 80f64.to_radians();
    let top = 100.0 * b.sin() - ORIGIN_Z * b.cos();
    let (_, _, warns, _) = generate(Rig::Head, 80.0, ORIGIN_Z + 40.0);
    let (h_mm, top_mm, fixture) = low(&warns).unwrap_or_else(|| panic!("no verdict under the table top: {warns:?}"));
    assert!((h_mm - 40.0).abs() < 1e-6, "the plane along the tool axis: {h_mm}");
    assert!(fixture && (top_mm - top).abs() < 1e-6, "the table's top face along the axis: {top_mm}, want {top}");
}
