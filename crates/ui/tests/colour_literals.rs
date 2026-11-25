//! The colour-literal guard (#1120): a colour in `crates/ui` is a named
//! token of the theme's table (`egui_app::theme`), never a literal
//! written at the draw site. A theme - ours or anyone's - can restyle
//! only what reads its tokens.
//!
//! The guard walks `crates/ui/src` and counts colour literals in each
//! file that is not a test file. Each count must equal its row in
//! `ALLOWED`:
//! - a file with a new literal, or a file not in the list, fails;
//! - a file whose literals moved to tokens fails until its row is
//!   lowered or removed, so the list only shrinks.
//!
//! A literal is a named constant (`Color32::WHITE`) or a constructor
//! whose arguments are all numbers written in place
//! (`Color32::from_rgb(245, 240, 250)`). A constructor over variables
//! (`from_rgb(r, g, b)`) converts a colour chosen elsewhere and does not
//! count. Colour data kept as plain numbers (a `[u8; 3]` const, a table
//! of tuples) is not seen; the #1120 family PRs move those by name.
//!
//! The permanent rows are the token tables themselves, the contrast
//! maths and a colour that only measures text. Each other row is a site
//! family that a later #1120 PR moves to tokens.
//! Provenance: our own test over our own source tree.
//!
//! Gate map (#1120): covers ui::egui_app::theme, ui::egui_app.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A colour written as a value at the call site. `Color32::TRANSPARENT`
/// is not in the list: it carries no hue for a theme to change.
const PATTERNS: &[&str] = &[
    "Color32::from_rgb(",
    "Color32::from_rgba_unmultiplied(",
    "Color32::from_rgba_premultiplied(",
    "Color32::from_gray(",
    "Color32::from_black_alpha(",
    "Color32::from_white_alpha(",
    "Color32::from_hex(",
    "Rgba::from_rgb(",
    "Color32::WHITE",
    "Color32::BLACK",
    "Color32::GRAY",
    "Color32::LIGHT_GRAY",
    "Color32::DARK_GRAY",
    "Color32::RED",
    "Color32::LIGHT_RED",
    "Color32::DARK_RED",
    "Color32::GREEN",
    "Color32::LIGHT_GREEN",
    "Color32::DARK_GREEN",
    "Color32::BLUE",
    "Color32::LIGHT_BLUE",
    "Color32::DARK_BLUE",
    "Color32::YELLOW",
    "Color32::LIGHT_YELLOW",
    "Color32::ORANGE",
    "Color32::GOLD",
    "Color32::BROWN",
    "Color32::KHAKI",
];

/// `(path under crates/ui/src, literal count, why)`. Keep it sorted by
/// path. A `keep:` row is not a colour choice for a theme; a `move`
/// row names the #1120 family PR that moves it to tokens and lowers it.
#[rustfmt::skip]
const ALLOWED: &[(&str, usize, &str)] = &[
    ("egui_app/colorvision.rs", 15, "move (#1120 verdict palettes): into the token table, values unchanged; the colour-vision maths stays"),
    ("egui_app/contrast.rs", 4, "keep: contrast maths (black or white text on a fill)"),
    ("egui_app/error_text.rs", 1, "move (#1120 status inks): the red fallback of an error line"),
    ("egui_app/file_drop.rs", 3, "move (#1120 viewport overlays): pick, handle and overlay inks"),
    ("egui_app/gen_overlay.rs", 6, "move (#1120 viewport overlays): pick, handle and overlay inks"),
    ("egui_app/shell.rs", 7, "move (#1120 viewport overlays): pick, handle and overlay inks"),
    ("egui_app/theme/tables.rs", 62, "keep: the token tables themselves"),
    ("egui_app/tool_body_lathe_page.rs", 3, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_body_page.rs", 4, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_holder_picker.rs", 2, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_library_dialog/assembly_group.rs", 2, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_library_dialog/holder_view.rs", 5, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_library_dialog/side_view.rs", 2, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_library_dialog/thumbs.rs", 2, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_model_view.rs", 1, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_preview.rs", 8, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/tool_profile_page.rs", 10, "move (#1120 tool drawings): tool, holder and profile previews"),
    ("egui_app/viewport_layout.rs", 1, "keep: the colour only measures text, it is not drawn"),
    ("egui_app/window_chrome.rs", 2, "move (#1120 window chrome): the close button hover and its ink"),
];

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// A test file: `tests.rs`, `*_tests.rs`, or anything under a `tests/` folder.
fn is_test_file(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name == "tests.rs"
        || name.ends_with("_tests.rs")
        || rel.starts_with("tests/")
        || rel.contains("/tests/")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        panic!("read {}", dir.display());
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// A number written in place: `255`, `0x2B`, `1.0`, `0.5_f32`.
fn is_number(arg: &str) -> bool {
    let a = arg.trim();
    a.starts_with(|c: char| c.is_ascii_digit())
        && a.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// The arguments of the call whose `(` ends at `open`, split at the
/// commas of its own depth; `None` when the call does not close.
fn call_args(text: &str, open: usize) -> Option<Vec<&str>> {
    let mut depth = 0usize;
    let mut start = open;
    let mut args = Vec::new();
    for (k, c) in text[open..].char_indices() {
        let at = open + k;
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    args.push(&text[start..at]);
                    return Some(args);
                }
            }
            ',' if depth == 1 => {
                args.push(&text[start..at]);
                start = at + 1;
            }
            _ => {}
        }
        if at == open {
            start = open + 1;
        }
    }
    None
}

/// The colour literals of one file. A constructor counts only when each
/// argument is a number written in place: `from_rgb(r, g, b)` converts a
/// colour chosen elsewhere and is not a literal. A named constant
/// (`Color32::WHITE`) always counts. A line comment does not count.
fn count(text: &str) -> usize {
    let code: String = text
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    PATTERNS
        .iter()
        .map(|pat| {
            code.match_indices(pat)
                .filter(|(i, _)| {
                    let end = i + pat.len();
                    if pat.ends_with('(') {
                        call_args(&code, end - 1).is_some_and(|args| {
                            // A trailing comma leaves one empty argument.
                            let args: Vec<&&str> =
                                args.iter().filter(|a| !a.trim().is_empty()).collect();
                            !args.is_empty() && args.iter().all(|a| is_number(a))
                        })
                    } else {
                        // `Color32::RED` must not count `Color32::REDDISH`-like names.
                        !code[end..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
                    }
                })
                .count()
        })
        .sum()
}

fn counts() -> BTreeMap<String, usize> {
    let root = src_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut out = BTreeMap::new();
    for f in files {
        let rel = f
            .strip_prefix(&root)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        if is_test_file(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let n = count(&text);
        if n > 0 {
            out.insert(rel, n);
        }
    }
    out
}

#[test]
fn no_colour_literal_outside_the_allowed_rows() {
    let found = counts();
    let allowed: BTreeMap<&str, usize> = ALLOWED.iter().map(|(p, n, _)| (*p, *n)).collect();
    let mut errors = Vec::new();
    for (rel, n) in &found {
        match allowed.get(rel.as_str()) {
            None => errors.push(format!(
                "{rel}: {n} colour literal(s), and the file has no row: read a theme token instead"
            )),
            Some(a) if n > a => errors.push(format!(
                "{rel}: {n} colour literals, the row allows {a}: read a theme token instead"
            )),
            Some(a) if n < a => errors.push(format!(
                "{rel}: {n} colour literals, the row says {a}: lower the row to {n}"
            )),
            Some(_) => {}
        }
    }
    for (rel, a) in &allowed {
        if !found.contains_key(*rel) {
            errors.push(format!(
                "{rel}: no colour literal left, the row says {a}: remove the row"
            ));
        }
    }
    let table: Vec<String> = found
        .iter()
        .map(|(p, n)| format!("    (\"{p}\", {n}, \"\"),"))
        .collect();
    assert!(
        errors.is_empty(),
        "{}\n\nToday's counts:\n{}",
        errors.join("\n"),
        table.join("\n")
    );
}

#[test]
fn the_rows_are_sorted_and_each_says_why() {
    let paths: Vec<&str> = ALLOWED.iter().map(|r| r.0).collect();
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        paths, sorted,
        "ALLOWED is sorted by path, one row for each file"
    );
    for (p, n, why) in ALLOWED {
        assert!(*n > 0, "{p}: a row with 0 literals is removed, not kept");
        assert!(!why.trim().is_empty(), "{p}: each row says why");
    }
}

#[test]
fn the_counter_sees_each_form_and_skips_comments() {
    let text = "let a = Color32::from_rgb(1, 2, 3);\n\
                // Color32::from_rgb(9, 9, 9) in a comment\n\
                let b = [Color32::WHITE, Color32::TRANSPARENT, Color32::from_gray(4)];\n\
                let c = Color32::RED; let d = Color32::REDDISH;\n\
                let e = Color32::from_rgb(\n    0x10,\n    0x20,\n    0x30,\n);\n\
                let f = Color32::from_rgba_unmultiplied(r, g, b, ALPHA);\n\
                let g = Color32::from_rgb(rgb.0, rgb.1, rgb.2);\n\
                let h = Color32::from_black_alpha(170);\n";
    assert_eq!(
        count(text),
        6,
        "from_rgb, WHITE, from_gray, RED, the split from_rgb and from_black_alpha; \
         not the comment, TRANSPARENT, REDDISH or a conversion of variables"
    );
}
