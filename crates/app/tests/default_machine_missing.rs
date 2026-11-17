//! The default machine file is the post's machine when a document mounts
//! none (#2319). A data directory whose posts folder is present but whose
//! `3axis-vmc.mch` is missing or malformed must refuse the post by the
//! file's name and write no program - never post with a null machine (no
//! limits, no coolant words). The spawns are real processes: the CLI exe
//! is copied beside a hand-made `assets/` folder, so the run-time lookup
//! reads that folder and no other.
//!
//! Gate map: covers app::posting, app::cli.

use camcraft_app::commands::{AddTool, CreateOperation, History, SetStock};
use camcraft_app::operation::{Height, OpParams, SrcRef};
use camcraft_app::project;
use camcraft_app::stock::StockShape;
use camcraft_app::tools::catalog;
use camcraft_app::world::new_world;
use flecs_ecs::prelude::World;

use std::path::{Path, PathBuf};
use std::process::Command;

/// Box stock and one facing op, no machine mounted: the post reads the
/// default machine file.
fn fixture() -> World {
    let world = new_world();
    let mut history = History::default();
    let cmd = SetStock::new(&world, StockShape::Box { dx: 60.0, dy: 60.0, dz: 20.0 }, Default::default());
    history.execute(&world, cmd.expect("box stock"));
    let tool = catalog().into_iter().find(|(n, _)| *n == "EM-0400-2FL").map(|(_, t)| t).expect("catalog entry");
    history.execute(&world, AddTool::new(&world, "EM-0400-2FL", tool).expect("catalog tool"));
    let params = OpParams::Face {
        holes: Default::default(),
        blend_turns: false,
        tilt: None,
        stepover: 3.0,
        clearance_z: Height::fixed(25.0),
        cut_z: Height::fixed(19.5),
        top_z: None,
        stepdown: None,
        feed: 1200.0,
        plunge_feed: 300.0,
        climb: true,
    };
    let cmd = CreateOperation::new(&world, "Face1", "EM-0400-2FL", SrcRef::Stock, params);
    history.execute(&world, cmd.expect("create"));
    world
}

/// A fresh folder with the CLI exe and `assets/posts` copied in, and an
/// `assets/machines` folder that `machines` fills.
fn install(tag: &str, machines: impl FnOnce(&Path)) -> PathBuf {
    let root = std::env::temp_dir().join(format!("camcraft-2319-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let posts = root.join("assets").join("posts");
    std::fs::create_dir_all(&posts).expect("posts folder");
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/posts");
    for entry in std::fs::read_dir(&src).expect("shipped posts") {
        let entry = entry.expect("entry");
        if entry.file_type().is_ok_and(|t| t.is_file()) {
            std::fs::copy(entry.path(), posts.join(entry.file_name())).expect("copy post");
        }
    }
    let mdir = root.join("assets").join("machines");
    std::fs::create_dir_all(&mdir).expect("machines folder");
    machines(&mdir);
    let exe = Path::new(env!("CARGO_BIN_EXE_camcraft-cli"));
    std::fs::copy(exe, root.join(exe.file_name().expect("exe name"))).expect("copy cli");
    root
}

/// Posts the fixture with the installed CLI: (exit code, stderr, program
/// text when one was written).
fn post_with(root: &Path) -> (i32, String, Option<String>) {
    let proj = root.join("job.camproj");
    project::save(&fixture(), &proj).expect("save fixture");
    let nc = root.join("out.nc");
    let exe = root.join(Path::new(env!("CARGO_BIN_EXE_camcraft-cli")).file_name().expect("exe name"));
    let o = Command::new(exe)
        .args(["process", proj.to_str().expect("utf8"), "-o", nc.to_str().expect("utf8")])
        .output()
        .expect("the CLI runs");
    let code = o.status.code().unwrap_or(-1);
    let err = String::from_utf8_lossy(&o.stderr).into_owned();
    (code, err, std::fs::read_to_string(&nc).ok())
}

fn assert_refused(tag: &str, machines: impl FnOnce(&Path)) {
    let root = install(tag, machines);
    let (code, err, program) = post_with(&root);
    let _ = std::fs::remove_dir_all(&root);
    assert!(program.is_none(), "{tag}: no program is written; got:\n{}", program.unwrap_or_default());
    assert_eq!(code, 1, "{tag}: the post refuses: {err}");
    assert!(err.contains("3axis-vmc.mch"), "{tag}: the refusal names the file: {err}");
    assert!(err.contains("default machine"), "{tag}: the refusal says which machine: {err}");
}

#[test]
fn a_missing_default_machine_file_refuses_the_post() {
    assert_refused("missing", |_| {});
}

#[test]
fn a_malformed_default_machine_file_refuses_the_post() {
    assert_refused("malformed", |dir| {
        std::fs::write(dir.join("3axis-vmc.mch"), "{ \"kinematics\": ").expect("write malformed file");
    });
}

#[test]
fn a_good_default_machine_file_still_posts() {
    // The control: the same install with the shipped file posts as before.
    let root = install("good", |dir| {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines/3axis-vmc.mch");
        std::fs::copy(src, dir.join("3axis-vmc.mch")).expect("copy default machine");
    });
    let (code, err, program) = post_with(&root);
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(code, 0, "{err}");
    assert!(program.is_some_and(|p| p.contains("G01")), "a program is written");
}
