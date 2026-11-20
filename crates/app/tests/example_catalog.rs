//! Example catalog contract (#2392, #2393): every case under
//! `examples/catalog/<family>/<case>/` carries its model generator/mesh,
//! `build.camscript`, `<case>.camproj`, `<case>.nc`, and `README.md`.
//! Each case opens, regenerates every operation, verifies clean, and posts
//! the committed `.nc` byte for byte; the seed case rebuilds end to end
//! from `build.camscript`. The turning case (#2394) also pins what its
//! program shows: the taper and the thread passes.
//!
//! Gate map (#2393): covers app::commands, app::model, app::operation,
//! app::posting, app::project, app::script, app::mcp, app::cli,
//! app::setup, app::world, post::post, sim::verify.
//! Provenance: clean-room test suite over CamCraft's own commands, script
//! host, and example catalog cases.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use camcraft_app::commands::{GenerateOperation, History};
use camcraft_app::operation::OpStatus;
use camcraft_app::{model, operation, posting, project, setup, world};

struct CatalogCase {
    family: String,
    name: String,
    dir: PathBuf,
}

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

fn discover_cases(catalog_root: &Path) -> Vec<CatalogCase> {
    assert!(
        catalog_root.is_dir(),
        "example catalog directory missing: {}",
        catalog_root.display()
    );
    let mut families: Vec<PathBuf> = fs::read_dir(catalog_root)
        .unwrap_or_else(|e| panic!("read {}: {e}", catalog_root.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    families.sort();

    let mut cases = Vec::new();
    for fam_dir in families {
        let family = fam_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        let mut children: Vec<PathBuf> = fs::read_dir(&fam_dir)
            .unwrap_or_else(|e| panic!("read {}: {e}", fam_dir.display()))
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        children.sort();
        for dir in children {
            let name = dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_owned();
            cases.push(CatalogCase { family: family.clone(), name, dir });
        }
    }
    cases
}

fn assert_case_layout(case: &CatalogCase) {
    let model_dir = case.dir.join("model");
    assert!(model_dir.is_dir(), "{}/{}: missing model/", case.family, case.name);
    let model_entries: Vec<_> = fs::read_dir(&model_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", model_dir.display()))
        .filter_map(Result::ok)
        .collect();
    assert!(
        !model_entries.is_empty(),
        "{}/{}: model/ must hold generator and mesh",
        case.family,
        case.name
    );
    for req in [
        "build.camscript".to_owned(),
        format!("{}.camproj", case.name),
        format!("{}.nc", case.name),
        "README.md".to_owned(),
    ] {
        let p = case.dir.join(&req);
        assert!(p.is_file(), "{}/{}: missing {}", case.family, case.name, req);
        assert!(
            fs::metadata(&p).map(|m| m.len()).unwrap_or(0) > 0,
            "{}/{}: empty {}",
            case.family,
            case.name,
            req
        );
    }
}

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap_or_else(|e| panic!("mkdir {}: {e}", dst.display()));
    for entry in fs::read_dir(src).unwrap_or_else(|e| panic!("read {}: {e}", src.display())) {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &target);
        } else {
            fs::copy(&path, &target)
                .unwrap_or_else(|e| panic!("copy {} -> {}: {e}", path.display(), target.display()));
        }
    }
}

// Portable examples embed their meshes. A relative source link makes the UI
// reopen a file from the process directory instead of the case directory.
fn assert_portable_models(case: &CatalogCase, w: &flecs_ecs::prelude::World) {
    let names: &[&str] = match (case.family.as_str(), case.name.as_str()) {
        ("2d", "contour-tabs") => &["contour-tabs"],
        ("turning", "cone-thread") => &["cone-blank", "cone-thread"],
        _ => return,
    };
    for name in names {
        let entity = model::find_by_name(w, name).expect("catalog model must be present");
        assert!(
            camcraft_app::model_source::of(w, &entity).is_none(),
            "{}/{}: {name} must open without a link to the script working directory",
            case.family, case.name
        );
        assert!(
            model::facets_of(w, &entity).is_some_and(|facets| !facets.is_empty()),
            "{}/{}: {name} must keep its embedded mesh", case.family, case.name
        );
    }
}

#[test]
fn every_catalog_case_opens_regenerates_verifies_and_posts_byte_for_byte() {
    let cases = discover_cases(&repo("examples/catalog"));
    assert!(
        cases.iter().any(|c| c.family == "2d" && c.name == "first-part"),
        "seed case 2d/first-part must be present under examples/catalog/"
    );
    for case in &cases {
        assert_case_layout(case);
        let label = format!("{}/{}", case.family, case.name);
        let proj_path = case.dir.join(format!("{}.camproj", case.name));
        let nc_path = case.dir.join(format!("{}.nc", case.name));

        let w = world::new_world();
        let notes = project::load_noted(&w, &proj_path)
            .unwrap_or_else(|e| panic!("{label}: load {}: {e}", proj_path.display()));
        assert!(notes.is_empty(), "{label}: unexpected load notes: {notes:?}");
        assert_portable_models(case, &w);

        let ops = setup::operation_order(&w);
        assert!(!ops.is_empty(), "{label}: project has no operations");

        let mut history = History::default();
        for (op_name, _) in &ops {
            let cmd = GenerateOperation::new(&w, op_name)
                .unwrap_or_else(|e| panic!("{label}: regenerate {op_name}: {e}"));
            history.execute(&w, cmd);
        }

        for (op_name, entity) in setup::operation_order(&w) {
            let status = operation::status_of(&w, &entity);
            assert_eq!(status, OpStatus::Ok, "{label}: {op_name} status after regenerate");
            if let Some(tp) = model::toolpath_of(&w, &entity) {
                let blocking: Vec<String> = tp
                    .warnings
                    .iter()
                    .filter(|warn| warn.blocks_post())
                    .map(ToString::to_string)
                    .collect();
                assert!(
                    blocking.is_empty(),
                    "{label}: {op_name} holds blocking warnings: {blocking:?}"
                );
            }
        }

        let report = posting::verification_report(&w);
        assert!(
            report.problems.is_empty(),
            "{label}: verification failed: {:?}",
            report.problems
        );

        let posted = posting::post_setup_unwritten(&w)
            .unwrap_or_else(|e| panic!("{label}: post failed: {e}"));
        let want = fs::read(&nc_path)
            .unwrap_or_else(|e| panic!("{label}: read {}: {e}", nc_path.display()));
        assert_eq!(
            posted.program.as_bytes(),
            want.as_slice(),
            "{label}: regenerated NC differs from {}",
            nc_path.display()
        );
    }
}

#[test]
fn every_catalog_case_rebuilds_from_camscript() {
    let cases = discover_cases(&repo("examples/catalog"));
    assert!(!cases.is_empty(), "at least one catalog case required");

    let scratch = std::env::temp_dir().join(format!("camcraft-catalog-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);

    for case in &cases {
        let label = format!("{}/{}", case.family, case.name);
        let work = scratch.join(&case.family).join(&case.name);
        copy_dir_all(&case.dir, &work);

        let proj = work.join(format!("{}.camproj", case.name));
        let nc = work.join(format!("{}.nc", case.name));
        let _ = fs::remove_file(&proj);
        let _ = fs::remove_file(&nc);

        let output = Command::new(env!("CARGO_BIN_EXE_camcraft-cli"))
            .arg("script")
            .arg("build.camscript")
            .current_dir(&work)
            .output()
            .unwrap_or_else(|e| panic!("{label}: run camcraft-cli: {e}"));

        assert!(
            output.status.success(),
            "{label}: build.camscript failed (exit {:?}):\nstdout: {}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert!(proj.is_file(), "{label}: build.camscript did not write {}", proj.display());
        assert!(nc.is_file(), "{label}: build.camscript did not write {}", nc.display());

        // Load outside the script's directory. Pin the rebuilt example too;
        // checking the committed project alone cannot catch a linked rebuild.
        if matches!((case.family.as_str(), case.name.as_str()),
            ("2d", "contour-tabs") | ("turning", "cone-thread")) {
            let w = world::new_world();
            let notes = project::load_noted(&w, &proj)
                .unwrap_or_else(|e| panic!("{label}: load rebuilt project: {e}"));
            assert!(notes.is_empty(), "{label}: rebuilt project load notes: {notes:?}");
            assert_portable_models(case, &w);
        }

        let want_nc = fs::read(case.dir.join(format!("{}.nc", case.name)))
            .unwrap_or_else(|e| panic!("{label}: read committed .nc: {e}"));
        let got_nc = fs::read(&nc)
            .unwrap_or_else(|e| panic!("{label}: read rebuilt .nc: {e}"));
        assert_eq!(
            got_nc, want_nc,
            "{label}: build.camscript NC differs from committed .nc"
        );
    }

    let _ = fs::remove_dir_all(&scratch);
}

/// The value of an address word (`X`, `Z`) in one NC block.
fn word(block: &str, address: char) -> Option<f64> {
    block
        .split_whitespace()
        .find_map(|w| w.strip_prefix(address))
        .and_then(|v| v.parse().ok())
}

/// The blocks of one operation: from its `(OPERATION: <name> ` comment
/// to the next operation comment.
fn section<'a>(program: &'a str, op: &str) -> Vec<&'a str> {
    let head = format!("(OPERATION: {op} ");
    program
        .lines()
        .skip_while(|l| !l.starts_with(&head))
        .skip(1)
        .take_while(|l| !l.starts_with("(OPERATION: "))
        .collect()
}

/// #2394: the turning case shows a spun profile - a cone with a threaded
/// end. The committed program holds the six operations in order, the
/// 20 mm to 10 mm taper over 40 mm in the finish ride, and the M10x1.5
/// thread passes to the 8.16 mm root on the Fanuc-style lathe post.
#[test]
fn turning_cone_thread_case_posts_the_taper_and_the_thread() {
    let dir = repo("examples/catalog/turning/cone-thread");
    let case = CatalogCase { family: "turning".into(), name: "cone-thread".into(), dir: dir.clone() };
    assert!(dir.is_dir(), "catalog case turning/cone-thread is missing: {}", dir.display());
    assert_case_layout(&case);
    let program = fs::read_to_string(dir.join("cone-thread.nc")).expect("read cone-thread.nc");

    assert!(program.contains("G18"), "a lathe program states the ZX plane");
    assert!(!program.lines().any(|l| word(l, 'Y').is_some()), "a 2-axis lathe program has no Y word");
    let kinds: Vec<&str> = program
        .lines()
        .filter(|l| l.starts_with("(OPERATION: "))
        .filter_map(|l| l.split(" - ").nth(1))
        .collect();
    assert_eq!(
        kinds,
        ["FACING", "PROFILE ROUGH", "PROFILE FINISH", "GROOVE FINISH", "THREADING", "PARTING"],
        "face, rough profile, finish profile, the neck, the thread, part-off - in this order"
    );

    // The taper: every finish block on the cone cuts X and Z together at
    // 10 mm of diameter over 40 mm of length (-0.25), and the blocks span
    // the cone between the shoulder (Z20) and the thread diameter (Z60).
    let finish = section(&program, "Finish profile");
    let cone: Vec<(f64, f64)> = finish
        .iter()
        .filter(|l| l.starts_with("G01"))
        .filter_map(|l| Some((word(l, 'X')?, word(l, 'Z')?)))
        .filter(|&(x, z)| x < 20.0 && (20.0..=58.5).contains(&z))
        .collect();
    assert!(cone.len() >= 2, "the finish ride has no blocks on the cone: {finish:?}");
    let span = cone[cone.len() - 1].1 - cone[0].1;
    assert!(span > 30.0, "the cone blocks span {span} mm of the 40 mm taper: {cone:?}");
    for w in cone.windows(2) {
        let ((x0, z0), (x1, z1)) = (w[0], w[1]);
        let slope = (x1 - x0) / (z1 - z0);
        assert!((slope + 0.25).abs() < 0.005, "X{x0} Z{z0} to X{x1} Z{z1} is off the taper: {slope}");
        assert!(x0 < 20.0 && x1 > 10.0, "X{x0} to X{x1} is outside the 20 to 10 mm cone");
    }

    // The thread: G32 passes at the 1.5 mm lead, the last one at the
    // M10x1.5 root diameter, from the end face side down to the neck.
    let thread = section(&program, "Thread");
    assert!(thread.iter().any(|l| l.starts_with("(T0303 ") && l.contains("THREADING INSERT")), "{thread:?}");
    assert!(thread.iter().any(|l| l.starts_with("G32 ") && l.ends_with("F1.5")), "no G32 pass at lead 1.5: {thread:?}");
    assert!(thread.contains(&"G01 X8.16 Z77.4 F0.05"), "the last pass enters at the 8.16 mm root: {thread:?}");
    assert!(thread.contains(&"G32 Z63.9 F1.5"), "each pass ends at the neck, Z63.9: {thread:?}");
    assert!(!program.contains("G76"), "this case posts G32 passes, not the G76 cycle");

    // The neck and the part-off: the 3 mm blade, to X8 and to the axis.
    assert!(section(&program, "Neck").contains(&"G01 X8. Z60. F0.02"), "the neck is 8 mm at Z60");
    assert!(section(&program, "Part off").iter().any(|l| l.starts_with("G01 X0. Z13.5")), "the part-off reaches the axis");
}

/// The G word and the Z word of each motion block of a posted program.
fn motion_blocks(nc: &str) -> Vec<(&str, f64)> {
    nc.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let g = words.next().filter(|w| matches!(*w, "G00" | "G01" | "G02" | "G03"))?;
            let z = words.find_map(|w| w.strip_prefix('Z'))?.parse::<f64>().ok()?;
            Some((g, z))
        })
        .collect()
}

/// #2859: the case 2d/contour-tabs posts what its README says. Each cut
/// run has a lead-in arc and a linear lead-out. The levels are Z4, Z2 (the
/// tab top) and Z0. The Z0 level lifts over each of the four tabs.
#[test]
fn contour_tabs_case_posts_the_lead_moves_and_the_tab_lifts() {
    let nc_path = repo("examples/catalog/2d/contour-tabs/contour-tabs.nc");
    let nc = fs::read_to_string(&nc_path)
        .unwrap_or_else(|e| panic!("case 2d/contour-tabs must hold {}: {e}", nc_path.display()));
    let blocks = motion_blocks(&nc);

    // The outline is cut clockwise (G02 corners). Only a lead-in arc turns
    // the other way (G03).
    let lead_ins = |z: f64| blocks.iter().filter(|b| b.0 == "G03" && b.1 == z).count();
    assert_eq!(lead_ins(4.0), 1, "one lead-in arc on the whole ring at Z4");
    assert_eq!(lead_ins(2.0), 1, "one lead-in arc on the whole ring at Z2, the tab top");
    assert_eq!(lead_ins(0.0), 4, "one lead-in arc for each run between two tabs at Z0");

    // A lift goes from a cut level to a height above the stock top (Z6).
    // The block before it is the end of the linear lead-out.
    let lifts: Vec<usize> =
        (1..blocks.len()).filter(|&i| blocks[i - 1].1 <= 4.0 && blocks[i].1 > 6.0).collect();
    assert_eq!(lifts.len(), 6, "two whole rings and four tab runs each end with a lift");
    for &i in &lifts {
        assert_eq!(blocks[i - 1].0, "G01", "the block before lift {i} is the linear lead-out");
    }
    let tab_lifts = lifts.iter().filter(|&&i| blocks[i - 1].1 == 0.0).count();
    assert_eq!(tab_lifts, 4, "the Z0 level lifts four times: one lift for each tab");
    assert!(!blocks.iter().any(|b| b.1 < 0.0), "no block goes below the stock bottom (Z0)");
}

/// #2862: the hole-work example keeps the four hole operations and posts
/// drilling cycles plus a thread helix with simultaneous XY and Z motion.
#[test]
fn hole_work_case_posts_drilling_cycles_and_thread_helix() {
    let dir = repo("examples/catalog/2d/hole-work");
    let case = CatalogCase { family: "2d".into(), name: "hole-work".into(), dir };
    assert_case_layout(&case);
    let nc = fs::read_to_string(case.dir.join("hole-work.nc")).expect("read hole-work.nc");
    let kinds: Vec<_> = nc.lines().filter(|line| line.starts_with("(OPERATION: "))
        .filter_map(|line| line.split(" - ").nth(1)).collect();
    assert_eq!(kinds, ["DRILL", "BORE", "CIRCULAR", "DRILL", "THREAD"]);
    let section = |name: &str| -> Vec<&str> {
        let header = format!("(OPERATION: {name} ");
        nc.lines().skip_while(|line| !line.starts_with(&header)).skip(1)
            .take_while(|line| !line.starts_with("(OPERATION: ")).collect()
    };
    let bolt = section("Bolt circle");
    for xy in ["X0. Y30.", "X30. Y0.", "X0. Y-30.", "X-30. Y0."] {
        assert!(bolt.iter().any(|line| line.starts_with(&format!("G81 {xy} Z-2.803 "))),
            "missing bolt-circle drilling cycle at {xy}");
    }
    for name in ["Bore", "Circular"] {
        assert!(section(name).iter().any(|line| line.starts_with("G03 ")),
            "{name} must retain circular interpolation");
    }
    assert!(section("Tap drill").contains(&"G81 X0. Y0. Z4. R20. F80"));
    let thread = section("Thread");
    let helix: Vec<_> = thread.iter().filter(|line| {
        let words: Vec<_> = line.split_whitespace().collect();
        matches!(words.first(), Some(&"G02" | &"G03"))
            && ['X', 'Y', 'Z', 'I', 'J'].iter().all(|address|
                words.iter().any(|word| word.starts_with(*address)))
    }).collect();
    assert_eq!(helix.len(), 12, "three thread passes each have four helical blocks");
    let z = |line: &str| -> f64 {
        line.split_whitespace().find_map(|word| word.strip_prefix('Z'))
            .expect("helix Z").parse().expect("numeric helix Z")
    };
    for pass in helix.as_chunks::<4>().0 {
        assert_eq!(z(pass[0]), 8.313, "thread entry rises a quarter pitch");
        assert_eq!(z(pass[3]), 9.875, "thread exit rises another quarter pitch");
        assert!((z(pass[2]) - z(pass[1]) - 0.625).abs() < 0.001,
            "half a turn must rise half the 1.25 mm pitch");
    }
}
