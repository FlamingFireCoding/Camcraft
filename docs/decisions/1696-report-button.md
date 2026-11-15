# Decisions - issue #1696

## #1696 - the Report door: a prefilled GitHub issue and a clipboard screenshot

**What was decided.** A Report door on the status strip opens a small
window. It takes a short title, and its two buttons ("Report a bug",
"Request a feature") open the browser on a prefilled new-issue URL:
`https://github.com/NeoTech/Camcraft/issues/new?template=blank-issue.md`
with `title`, `body` and `labels` in the query. The body embeds the
Debug info report (#157) - same fields, one source. A "Copy screenshot"
button puts a PNG of the current document on the clipboard; the user
pastes it into the issue box and GitHub uploads it. The repository also
gained `blank-issue.md` and a `config.yml` that enables blank issues.

**Why.** The maintainer asked (issue #1696) for one button that prepares
a bug or feature report with a screenshot and the relevant data
attached. GitHub's prefill query is the only documented way to fill a
new issue from an app, and it works only on a blank template. egui's
`Context::open_url` and `Context::copy_image` exist in the pinned
egui 0.36.2 and are already plumbed by the shell's platform-output
handler, so no new dependency is needed.

**Considered and dropped.**
- Opening the existing issue FORMS (`bug.yml`, `feature.yml`) prefilled:
  GitHub ignores the query on issue forms (community discussion
  orgs/community#15477). The blank template plus the `labels` query
  reproduces their `bug` / `enhancement` labels.
- Attaching the PNG file from the app: a browser cannot receive a file
  from another program. The clipboard paste is the shortest path, and
  GitHub uploads a pasted picture on its own.
- The GPU `snapshot_rgba()` path for the picture: the viewport's own
  readback needs a device and is unused by the app. The CPU raster
  (`camcraft_app::raster`, the MCP `render_stock` path) works headless
  and deterministic.
- Writing the report body through a new command: the window writes only
  view state (an open flag, a title string), the same class as the
  Debug info window's state, so "Everything Is a Command" does not
  apply.

**Cost to change later.** The URL, the template name and the label names
are three constants in `report.rs`; a fork pointing at its own repo
changes one constant. If GitHub later lets a URL prefill an issue form,
the blank template can go and `template=` changes - no Rust change
beyond that constant.
