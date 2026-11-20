//! The path scrubber of the shared report (#2744): the Report door's text
//! leaves the machine, so each path in it shrinks to its last component.
//! A directory path can hold the user's name, a customer or a project.
//!
//! Provenance: our own; the path forms are the ones Windows (drive, UNC,
//! `\`) and POSIX (`/`, `~`) document. No vendor material consulted.

/// The word that stands for a path with no file name part (a root, `..`).
pub(crate) const NO_FILE_NAME: &str = "(no file name)";

/// The last component of a path, read with BOTH separators (`/` and `\`)
/// on every system: a project saved on Windows and opened on Linux keeps
/// its Windows paths, and `Path::file_name` there sees no separator in
/// them. Trailing separators are ignored (the last folder name results).
/// A root, a drive (`C:`), `.` or `..` gives [`NO_FILE_NAME`].
pub(crate) fn last_component(path: &str) -> String {
    let trimmed = path.trim_end_matches(['/', '\\']);
    let last = trimmed.rsplit(['/', '\\']).next().unwrap_or("");
    let is_drive = last.len() == 2 && last.ends_with(':') && last.starts_with(|c: char| c.is_ascii_alphabetic());
    if last.is_empty() || last == "." || last == ".." || is_drive {
        NO_FILE_NAME.to_owned()
    } else {
        last.to_owned()
    }
}

/// [`last_component`] of a `Path`; a non-UTF-8 name reads lossily.
pub(crate) fn file_name_only(path: &std::path::Path) -> String {
    last_component(&path.to_string_lossy())
}

/// Each path in `text` replaced by its last component; all other text as
/// it was. Line by line; words split at spaces.
pub(crate) fn scrub_paths(text: &str) -> String {
    text.split('\n').map(scrub_line).collect::<Vec<_>>().join("\n")
}

const LEAD: &[char] = &['"', '\'', '(', '[', '<', '{', '`'];
const TRAIL: &[char] = &['"', '\'', ')', ']', '>', '}', '`', ',', ';', ':', '.', '!', '?'];
/// Trailing marks that end a path: a path never runs past `open failed:
/// <path>: <reason>` into the reason.
const ENDS_PATH: &[char] = &[',', ';', ':', ')', ']', '>', '}', '"', '\'', '`'];

/// A word split into its leading marks, its body and its trailing marks.
fn split_marks(word: &str) -> (&str, &str, &str) {
    let body_start = word.len() - word.trim_start_matches(LEAD).len();
    let (lead, rest) = word.split_at(body_start);
    let body = rest.trim_end_matches(TRAIL);
    (lead, body, &rest[body.len()..])
}

fn has_separator(s: &str) -> bool {
    s.contains(['/', '\\'])
}

/// Does this word start a path? Rooted forms (POSIX `/`, `\`, UNC `\\`,
/// drive `C:\` or `C:/`, `~`, `./`, `../`), any `\`, or two or more `/`
/// (a relative POSIX path). One `/` alone (`mm/min`) is not a path.
fn starts_path(body: &str) -> bool {
    let b = body.as_bytes();
    let rooted = body.starts_with(['/', '\\'])
        || (body.starts_with('~') && has_separator(body))
        || body.starts_with("./")
        || body.starts_with("../")
        || body.starts_with(".\\")
        || body.starts_with("..\\")
        || (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/'));
    rooted || body.contains('\\') || body.matches('/').count() >= 2
}

fn scrub_line(line: &str) -> String {
    let words: Vec<&str> = line.split(' ').collect();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        let (lead, body, _) = split_marks(words[i]);
        if !starts_path(body) {
            out.push(words[i].to_owned());
            i += 1;
            continue;
        }
        // A folder name can hold spaces (`C:\Users\jane doe\x.camproj`):
        // the path runs on to the last later word that has a separator,
        // unless a word ends it first with a mark such as `:` or `,`.
        let mut end = i;
        if !ends_path(words[i]) {
            for (j, w) in words.iter().enumerate().skip(i + 1) {
                if has_separator(split_marks(w).1) {
                    end = j;
                }
                if ends_path(w) {
                    break;
                }
            }
        }
        let joined = words[i..=end].join(" ");
        let (_, body, trail) = split_marks(&joined[lead.len()..]);
        out.push(format!("{lead}{}{trail}", last_component(body)));
        i = end + 1;
    }
    out.join(" ")
}

fn ends_path(word: &str) -> bool {
    word.ends_with(ENDS_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_component_reads_both_separators_on_every_system() {
        // #2744 review: `Path::file_name` on Linux and macOS returns a
        // Windows path whole. This test runs the same on each system.
        assert_eq!(last_component(r"C:\Users\jane.doe\Parts\m.json"), "m.json");
        assert_eq!(last_component("/home/jane.doe/Parts/m.json"), "m.json");
        assert_eq!(last_component(r"\\server\share\jane\m.json"), "m.json");
        assert_eq!(last_component("C:/Users/jane/m.json"), "m.json");
        assert_eq!(last_component("m.json"), "m.json");
        // Trailing separators: the last folder name, the same on all systems.
        assert_eq!(last_component(r"C:\Users\jane\Parts\"), "Parts");
        assert_eq!(last_component("/home/jane/Parts/"), "Parts");
        // No file name part: the fixed word, never the path.
        for none in ["/", r"\", r"C:\", "C:/", "C:", "..", "../", r"..\", ".", ""] {
            assert_eq!(last_component(none), NO_FILE_NAME, "{none:?}");
        }
        assert_eq!(file_name_only(std::path::Path::new("/a/b/c.camproj")), "c.camproj");
    }

    #[test]
    fn scrub_paths_keeps_only_the_file_name_of_each_path() {
        let cases = [
            (r"open failed: C:\Users\someone\Documents\secret\part.camproj: not found", "open failed: part.camproj: not found"),
            ("open failed: /home/someone/secret/part.camproj: not found", "open failed: part.camproj: not found"),
            (r"saved C:\Users\jane doe\My Parts\part.camproj", "saved part.camproj"),
            ("written to ~/secret/out.nc", "written to out.nc"),
            (r"to \\server\share\someone\lib.json.", "to lib.json."),
            (r#"wrote "C:/Users/someone/x.json", 3 tools"#, r#"wrote "x.json", 3 tools"#),
            ("written to /home/someone/a.nc (12 tools)", "written to a.nc (12 tools)"),
            ("feed 1200 mm/min, ok", "feed 1200 mm/min, ok"),
            ("debug info copied to the clipboard", "debug info copied to the clipboard"),
            ("line one\n/home/someone/x.nc", "line one\nx.nc"),
        ];
        for (input, want) in cases {
            assert_eq!(scrub_paths(input), want, "{input:?}");
        }
    }
}
