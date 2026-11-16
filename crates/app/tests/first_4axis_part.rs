//! #1904: the "Your first 4-axis part" guide's demo job, built through the
//! app's own commands - `examples/first-4axis-part.camproj` and the program
//! it posts, `examples/first-4axis-part.nc`. A 3/4" round bar in the
//! microARC 4 chuck on a Tormach 770MX, axis along world X, the free end at
//! X0 (G54 X0 at the end face, Y0 Z0 on the A centreline): one shallow flat
//! on each of four faces (A-90, A0, A90, A180), then a cross slot on the A0
//! face with a second tool. Each operation is an indexed A, then an ordinary
//! 3-axis cut (3+1). The guide page quotes this program, and
//! `site/tests/test_first_4axis_part.py` holds the page to these two files.
//! Provenance: our own test over our own commands, machine file and post.
//!
//! Regenerate the project and the program after an intentional change:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-app --test first_4axis_part`
//!
//! Gate map (M4.4): covers app::posting (post_setup, verification_gate),
//! post::post (tormach.cps), app::operation (tilted Slot), app::project,
//! app::machine (check_document).

use camcraft_app::commands::{
    AddTool, CreateOperation, GenerateOperation, History, ImportModel, LoadMachine, SelectPost,
    SetStock, SetUnits,
};
use camcraft_app::geo2d::Point2;
use camcraft_app::geometry::Point3;
use camcraft_app::machining3d::Entry3D;
use camcraft_app::operation::{Height, HeightReference, OpParams, SrcRef, Tilt};
use camcraft_app::schema::{Location, PostRef};
use camcraft_app::stock::StockShape;
use camcraft_app::tools::{Tool, ToolKind, catalog};
use camcraft_app::units::Unit;
use camcraft_app::{machine, posting, project};
use flecs_ecs::prelude::World;
use std::path::{Path, PathBuf};

const IN: f64 = 25.4;
/// 3/4" bar, 1.5" sticking out of the chuck.
const R: f64 = 0.375 * IN;
const STICK: f64 = 1.5 * IN;
const MACHINE: &str = "Tormach 770MX + microARC 4";
const PROJECT: &str = "examples/first-4axis-part.camproj";
const PROGRAM: &str = "examples/first-4axis-part.nc";
const MODEL: &str = "examples/models/first-4axis-part.obj";
/// The five operations in program order, with the A word each one indexes to.
const OPS: [(&str, &str); 5] = [
    ("Flat A-90", "G0 A-90"),
    ("Flat A0", "G0 A0"),
    ("Flat A90", "G0 A90"),
    ("Flat A180", "G0 A180"),
    ("Cross slot A0", "G0 A0"),
];

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn ok<T, E: std::fmt::Debug>(r: Result<T, E>, what: &str) -> T {
    r.unwrap_or_else(|e| panic!("{what}: {e:?}"))
}

/// A tool as the guide makes it with New tool: the editor starts from the
/// catalog's first entry (an end mill), and the guide types the sizes in.
fn tool(dia_in: f64, flute_in: f64, rpm: f64, feed_ipm: f64) -> Tool {
    let (_, mut t) = catalog().into_iter().next().expect("the catalog's first entry");
    assert_eq!(t.kind, ToolKind::EndMill, "the New tool starter is an end mill");
    t.diameter = dia_in * IN;
    t.shank_d = t.diameter;
    t.flute_len = flute_in * IN;
    t.rpm = rpm;
    t.feed = feed_ipm * IN;
    t
}

/// The index of one face: the tool axis letters and the plane origin on
/// the A centreline at the bar's free end.
fn index(a_deg: f64, b_deg: f64) -> Option<Tilt> {
    Some(Tilt { a_deg, b_deg, origin: [0.0; 3] })
}

/// A slot typed as the guide types it, lengths in inches: the two ends in
/// the tilted plane's own X and Y, the width, the depth below the bar's
/// skin and the stepdown. The heights are the page's "Top of stock" plus an
/// offset. The stock box is centred on the A centreline, so its top is
/// 0.375" from the plane origin along the tool axis on every face.
#[allow(clippy::too_many_arguments)]
fn slot(a: (f64, f64), b: (f64, f64), width: f64, depth: f64, stepdown: f64, feed: f64, plunge: f64, tilt: Option<Tilt>) -> OpParams {
    let above = |offset_in: f64| Height::of(HeightReference::TopOfStock, offset_in * IN);
    OpParams::Slot {
        a: Point2::new(a.0 * IN, a.1 * IN),
        b: Point2::new(b.0 * IN, b.1 * IN),
        width: width * IN,
        stock_to_leave: 0.0,
        clearance_z: above(0.5),
        retract_z: None,
        top_z: above(0.0),
        bottom_z: above(-depth),
        stepdown: stepdown * IN,
        stepover: 3.0, // the Slot default; one pass as wide as the tool has no stepover
        feed: feed * IN,
        plunge_feed: plunge * IN,
        climb: true,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        links: camcraft_app::links::Links::PLANAR,
        tilt,
        extend_start: 0.0,
        extend_end: 0.0,
    }
}

/// A flat 0.015" deep along the bar, one pass of the 1/4" tool with its
/// centre from X-0.75 to X-0.25. `a` and `b` are the ends in the plane.
fn flat(a: (f64, f64), b: (f64, f64), tilt: Option<Tilt>) -> OpParams {
    slot(a, b, 0.25, 0.015, 0.015, 15.0, 4.0, tilt)
}

/// The document: machine, unit, post, stock, model, tools, the five ops.
fn build() -> World {
    let world = camcraft_app::world::new_world();
    let mut h = History::default();
    h.execute(&world, ok(LoadMachine::builtin(&world, MACHINE), "machine"));
    h.execute(&world, ok(SetUnits::new(&world, Unit::Inches), "inch document"));
    h.execute(&world, ok(SelectPost::new(&world, PostRef::Named("Tormach PathPilot".into())), "post"));
    h.execute(&world, ok(ImportModel::new(&repo(MODEL)), "model"));
    // The box the round bar fits in: it touches the bar on the four faces
    // the job cuts, so each cut starts at the real surface.
    let stock = SetStock::new(
        &world,
        StockShape::Box { dx: STICK, dy: 2.0 * R, dz: 2.0 * R },
        Location { pos: Point3::new(-STICK / 2.0, 0.0, -R), ..Location::default() },
    );
    h.execute(&world, ok(stock, "stock"));
    h.execute(&world, ok(AddTool::new(&world, "EM 1/4", tool(0.25, 0.75, 6000.0, 15.0)), "T1"));
    h.execute(&world, ok(AddTool::new(&world, "EM 1/8", tool(0.125, 0.375, 8000.0, 8.0)), "T2"));
    let along = ((-0.75, 0.0), (-0.25, 0.0));
    let ops: [(&str, &str, OpParams); 5] = [
        (OPS[0].0, "EM 1/4", flat(along.0, along.1, index(-90.0, 0.0))),
        (OPS[1].0, "EM 1/4", flat(along.0, along.1, None)),
        (OPS[2].0, "EM 1/4", flat(along.0, along.1, index(90.0, 0.0))),
        // The face underneath is tool axis A0 B180. Plane X stays along machine
        // X (the A rotary axis), so all four flats use the same ends (#2133).
        (OPS[3].0, "EM 1/4", flat(along.0, along.1, index(0.0, 180.0))),
        (OPS[4].0, "EM 1/8", slot((-0.5, -0.16), (-0.5, 0.16), 0.125, 0.030, 0.010, 8.0, 2.0, None)),
    ];
    for (name, t, p) in ops {
        h.execute(&world, ok(CreateOperation::new(&world, name, t, SrcRef::Stock, p), name));
        h.execute(&world, ok(GenerateOperation::new(&world, name), name));
    }
    world
}

fn post(world: &World, tag: &str) -> String {
    let out = std::env::temp_dir().join(format!("camcraft_first_4axis_{tag}_{}.nc", std::process::id()));
    let posted = posting::post_setup(world, &out);
    let _ = std::fs::remove_file(&out);
    posted.unwrap_or_else(|e| panic!("the first 4-axis part must post: {e}")).program
}

#[test]
fn the_first_4axis_part_posts_its_indexed_program() {
    let world = build();
    let nc = post(&world, "built");
    let golden = repo(PROGRAM);
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(&golden, &nc).expect("write the example program");
        project::save(&world, &repo(PROJECT)).expect("save the example project");
    } else {
        let want = std::fs::read_to_string(&golden).expect("the example program");
        assert_eq!(nc, want, "{} is stale; regenerate with WRITE_GOLDEN=1", golden.display());
    }
    // One index for each operation, in order, each alone after Z goes up.
    let lines: Vec<&str> = nc.lines().collect();
    let idx: Vec<&str> = lines.iter().copied().filter(|l| l.starts_with("G0 A")).collect();
    assert_eq!(idx, OPS.map(|(_, a)| a), "{nc}");
    for (i, l) in lines.iter().enumerate() {
        if l.starts_with("G0 A") {
            assert_eq!(lines[i - 1], "G53 G0 Z0", "{nc}");
        }
        let words = || l.split(' ').filter(|w| !w.is_empty());
        if !l.starts_with('(') {
            assert!(!(l.starts_with("G1") && words().any(|w| w.starts_with('A'))), "no A on a cut: {l}");
            assert!(!words().any(|w| w.starts_with('B') || w.starts_with('C')), "no B or C word: {l}");
        }
    }
    // Each cut stays within 0.031" of the bar's skin (Z is the radius after
    // each index) and inside the 1.5" stick-out.
    for l in lines.iter().filter(|l| l.starts_with("G1 ")) {
        for t in l.split(' ') {
            let v: Option<f64> = t.get(1..).and_then(|s| s.parse().ok());
            match (t.chars().next(), v) {
                (Some('Z'), Some(z)) => assert!((0.375 - 0.0301..=0.375 + 0.2).contains(&z), "{l}"),
                (Some('X'), Some(x)) => assert!((-1.5..=0.0).contains(&x), "{l}"),
                (Some('Y'), Some(y)) => assert!((-0.17..=0.17).contains(&y), "{l}"),
                _ => {}
            }
        }
    }
    assert!(nc.starts_with("(Tormach 770MX + microARC 4)\nG20 "), "an inch PathPilot program: {nc}");
    assert_eq!(lines.last().copied(), Some("M30"), "{nc}");
}

/// The four flats are the same cut in each face's own frame: after its
/// index, each one posts the same X Y Z lines. This is what "3+1" means,
/// and it holds the guide's statement about the plane X and Y of the face
/// underneath (A0 B180).
#[test]
fn each_flat_posts_the_same_cut_after_its_index() {
    let nc = post(&build(), "flats");
    let sections: Vec<&str> = nc.split("(OPERATION: ").skip(1).collect();
    assert_eq!(sections.len(), 5, "{nc}");
    let cut = |s: &str| -> Vec<String> {
        s.lines().skip_while(|l| !l.starts_with("G0 A")).skip(1).filter(|l| l.starts_with("G1 ")).map(str::to_owned).collect()
    };
    let first = cut(sections[0]);
    assert_eq!(first.len(), 4, "a plunge and a pass at the skin, then at the flat: {first:?}");
    for (i, s) in sections.iter().enumerate().take(4) {
        assert!(s.starts_with(OPS[i].0), "section {i} is {}", OPS[i].0);
        assert_eq!(cut(s), first, "{} cuts as {} does", OPS[i].0, OPS[0].0);
    }
    let under = index(0.0, 180.0).and_then(|t| t.frame()).expect("the A0 B180 frame");
    let p = under.to_local(Point3::new(-0.75 * IN, 0.0, -R));
    assert!((p.x + 0.75 * IN).abs() < 1e-9 && p.y.abs() < 1e-9 && (p.z - R).abs() < 1e-9, "{p:?}");
}

/// The committed project is the built document: it loads on the shipped
/// machine and post, passes the machine check and the post gate, and
/// posts the committed program.
#[test]
fn the_committed_project_posts_the_committed_program() {
    let world = camcraft_app::world::new_world();
    let notes = ok(project::load_noted(&world, &repo(PROJECT)), "load the example project");
    assert!(notes.is_empty(), "the example loads without migration notes: {notes:?}");
    let (_, mch) = machine::of(&world);
    let mch = mch.expect("the mounted machine");
    assert_eq!(mch.model, MACHINE);
    assert_eq!(posting::post_of(&world), PostRef::Named("Tormach PathPilot".into()));
    let names: Vec<String> = camcraft_app::setup::operation_order(&world).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, OPS.map(|(n, _)| n));
    let check = ok(machine::check_document(&world, &mch), "the machine check");
    assert!(check.is_ok(), "{:?}", check.problems);
    let gate = posting::verification_gate(&world);
    assert!(gate.is_empty(), "the post gate is silent: {gate:?}");
    let want = std::fs::read_to_string(repo(PROGRAM)).expect("the example program");
    assert_eq!(post(&world, "loaded"), want, "{PROJECT} posts {PROGRAM}");
}
