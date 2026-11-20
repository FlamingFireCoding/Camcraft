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
