//! The post and machine lists as the folders hold them (#2380): which
//! shipped names have their file now, and which other `.cps` / `.mch`
//! files lie in the folder. The registry tables ([`BUILTINS`],
//! [`MACHINES`]) only say which name goes with which file; a list built
//! from them alone shows a removed file and hides an added one.
//!
//! Provenance: our own rule, specced from issue #2380 (the list follows the
//! folder at start and at each refresh); no outside source.
//!
//! The scan is read at the first call and again at [`rescan`] (the
//! Refresh buttons), never per frame. A scan reads names only - it opens no
//! file - so a malformed file cannot break it or hide another file; the
//! entry that shows it later carries its own refusal. A name that a
//! document needs and that has no file is still refused by name when the
//! text is read (`builtin_post_text`, `builtin_machine_text`, #2319).

use super::files::{AssetFile, BUILTINS, MACHINES};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// One folder as read: the table names that have a file, and the other files.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FolderScan {
    /// Table names whose file is in the folder, in table order.
    pub present: Vec<&'static str>,
    /// Files the table does not name: (list name = file stem, path), by file name.
    pub extra: Vec<(String, PathBuf)>,
    /// Why the folder did not read, when it did not (the lists stay empty).
    pub problem: Option<String>,
}

/// Scan `dir` for `ext` files against `table`. A file in `hidden` is neither
/// a table entry nor an extra (the default machine's file: the post's own
/// fallback, not a list entry, as before). Never panics, never opens a file.
#[must_use]
pub fn scan_folder(dir: &Path, ext: &str, table: &[(&'static str, AssetFile)], hidden: &[AssetFile]) -> FolderScan {
    let list = match camcraft_post::data_dir::list_files(dir, ext) {
        Ok(l) => l,
        Err(why) => return FolderScan { problem: Some(why), ..FolderScan::default() },
    };
    for note in &list.skipped {
        tracing::warn!("{note}");
    }
    let named = |f: &str| table.iter().find(|(_, file)| file.0.eq_ignore_ascii_case(f));
    let present = table.iter().filter(|(_, f)| list.names.iter().any(|n| n.eq_ignore_ascii_case(f.0))).map(|(n, _)| *n).collect();
    let extra = list
        .names
        .iter()
        .filter(|n| named(n).is_none() && !hidden.iter().any(|h| h.0.eq_ignore_ascii_case(n)))
        .map(|n| (Path::new(n).file_stem().and_then(std::ffi::OsStr::to_str).unwrap_or(n).to_owned(), dir.join(n)))
        .collect();
    FolderScan { present, extra, problem: None }
}

type Scans = (FolderScan, FolderScan);
static SCANS: Mutex<Option<Scans>> = Mutex::new(None);

#[cfg(target_arch = "wasm32")]
fn read_process() -> Scans {
    // The browser build has no folder: every table entry is compiled in.
    let all = |t: &[(&'static str, AssetFile)]| FolderScan { present: t.iter().map(|(n, _)| *n).collect(), ..FolderScan::default() };
    (all(BUILTINS), all(MACHINES))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_process() -> Scans {
    let one = |files: &camcraft_post::data_dir::DataFiles, ext: &str, table: &[(&'static str, AssetFile)], hidden: &[AssetFile]| match files.dir() {
        Ok(dir) => scan_folder(dir, ext, table, hidden),
        Err(why) => FolderScan { problem: Some(why), ..FolderScan::default() },
    };
    let posts = one(super::files::post_dir(), "cps", BUILTINS, &[]);
    let machines = one(super::files::machine_dir(), "mch", MACHINES, &[super::files::DEFAULT_MCH_FILE]);
    for (kind, s) in [("posts", &posts), ("machines", &machines)] {
        match &s.problem {
            Some(why) => tracing::error!("{kind} folder: {why}"),
            None => tracing::info!("{kind} folder: {} shipped, {} other file(s)", s.present.len(), s.extra.len()),
        }
    }
    (posts, machines)
}

fn with_scans<T>(f: impl FnOnce(&Scans) -> T) -> T {
    let mut g = SCANS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    f(g.get_or_insert_with(read_process))
}

/// The post folder as last read (read at first use).
#[must_use]
pub fn post_scan() -> FolderScan {
    with_scans(|s| s.0.clone())
}

/// The machine folder as last read (read at first use).
#[must_use]
pub fn machine_scan() -> FolderScan {
    with_scans(|s| s.1.clone())
}

/// Read both folders again (the Refresh buttons call this).
pub fn rescan() {
    *SCANS.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(read_process());
}
