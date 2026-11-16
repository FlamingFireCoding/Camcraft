//! Pins of the run-time data directory lookup (#2282).

use super::*;

#[cfg(windows)]
const ROOT: &str = "C:";
#[cfg(not(windows))]
const ROOT: &str = "";

fn p(s: &str) -> PathBuf {
    PathBuf::from(format!("{ROOT}{s}"))
}

fn dirs(accept: &[&str]) -> impl Fn(&Path) -> bool {
    let set: Vec<PathBuf> = accept.iter().map(|s| p(s)).collect();
    move |q: &Path| set.iter().any(|d| d == q)
}

const EXE: &str = "/opt/cc/target/release/camcraft-ui.exe";
const USER: &str = "/home/u/.local/share/camcraft";
const MANIFEST: &str = "/src/camcraft/crates/post";

/// A fresh, empty scratch directory for one test.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("camcraft-data-dir-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn the_order_is_exe_user_dev_manifest() {
    let got = candidates("posts", Some(&p(EXE)), Some(&p(USER)), Some(&p(MANIFEST)));
    assert_eq!(
        got,
        vec![
            p("/opt/cc/target/release/assets/posts"),
            p("/home/u/.local/share/camcraft/assets/posts"),
            p("/opt/cc/assets/posts"),
            p("/src/camcraft/assets/posts"),
        ]
    );
}

#[test]
fn beside_the_exe_wins_over_the_user_copy() {
    let both = dirs(&["/opt/cc/target/release/assets/machines", "/home/u/.local/share/camcraft/assets/machines"]);
    let got = resolve("machines", Some(&p(EXE)), Some(&p(USER)), None, both);
    assert_eq!(got, Ok(p("/opt/cc/target/release/assets/machines")));
}

#[test]
fn the_macos_bundle_resources_are_found() {
    let exe = p("/Applications/CamCraft.app/Contents/MacOS/camcraft-ui");
    let want = "/Applications/CamCraft.app/Contents/Resources/assets/posts";
    assert_eq!(resolve("posts", Some(&exe), None, None, dirs(&[want])), Ok(p(want)));
}

#[test]
fn the_user_data_root_follows_each_platform() {
    let env = |k: &str| match k {
        "APPDATA" => Some(OsString::from(p("/Users/u/AppData/Roaming"))),
        "HOME" => Some(OsString::from(p("/home/u"))),
        _ => None,
    };
    assert_eq!(user_data_for("windows", env), Some(p("/Users/u/AppData/Roaming/camcraft")));
    assert_eq!(user_data_for("linux", env), Some(p("/home/u/.local/share/camcraft")));
    assert_eq!(user_data_for("macos", env), Some(p("/home/u/Library/Application Support/camcraft")));
    assert_eq!(user_data_for("linux", |_| None), None);
}

/// Done means (#2282): a test pins the error for a missing directory.
#[test]
fn a_missing_directory_is_a_named_error_not_a_crash() {
    let why = resolve("posts", Some(&p(EXE)), Some(&p(USER)), None, dirs(&[])).expect_err("nothing exists");
    assert!(why.starts_with("no posts folder found"), "{why}");
    assert!(why.contains("assets/posts directory beside the executable"), "{why}");
    let first = candidates("posts", Some(&p(EXE)), None, None)[0].display().to_string();
    assert!(why.contains(&first), "{why}");
    let files = DataFiles::new(Err(why.clone()));
    assert_eq!(files.read("fanuc.cps").map(|_| ()), Err(why), "a read names the missing directory");
    let none = resolve("machines", None, None, None, dirs(&[])).expect_err("no exe path");
    assert!(none.contains("nowhere"), "{none}");
}

#[test]
fn a_missing_file_names_the_file_and_the_directory() {
    let dir = scratch("missing-file");
    let files = DataFiles::new(Ok(dir.clone()));
    let why = files.read("gone.cps").expect_err("no such file");
    assert!(why.starts_with("gone.cps in "), "{why}");
    assert!(why.contains(&dir.display().to_string()), "{why}");
    let _ = std::fs::remove_dir_all(dir);
}

/// Done means (#2282): an edited file on disk changes what is loaded,
/// and the read is cached (one read per file, never per frame).
#[test]
fn an_edited_file_on_disk_is_what_loads_and_the_read_is_cached() {
    let dir = scratch("edited");
    std::fs::write(dir.join("mine.cps"), "// edited by the user\n").expect("write");
    let files = DataFiles::new(Ok(dir.clone()));
    assert_eq!(&*files.read("mine.cps").expect("reads"), "// edited by the user\n");
    std::fs::write(dir.join("mine.cps"), "// changed again\n").expect("rewrite");
    assert_eq!(&*files.read("mine.cps").expect("cached"), "// edited by the user\n", "a second read comes from the cache");
    assert_eq!(&*DataFiles::new(Ok(dir.clone())).read("mine.cps").expect("fresh"), "// changed again\n");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_file_that_is_not_text_is_refused_by_name() {
    let dir = scratch("binary");
    std::fs::write(dir.join("bad.mch"), [0xff_u8, 0xfe, 0x00]).expect("write");
    let why = DataFiles::new(Ok(dir.clone())).read("bad.mch").expect_err("not UTF-8");
    assert!(why.contains("bad.mch in") && why.contains("not UTF-8"), "{why}");
    let _ = std::fs::remove_dir_all(dir);
}

/// The dev run and the tests find the repository `assets` directory.
#[test]
fn the_dev_run_finds_the_repository_assets() {
    for (kind, file) in [("posts", "fanuc.cps"), ("machines", "vmc.mch")] {
        let files = DataFiles::from_process(kind);
        let dir = files.dir().unwrap_or_else(|why| panic!("{why}"));
        assert!(dir.join(file).is_file(), "{}", dir.display());
        assert!(!files.read(file).expect("reads").is_empty());
    }
}
