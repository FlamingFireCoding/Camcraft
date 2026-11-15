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

## #1696 follow-up - the screenshot's ground grid (GUI test finding)

**What was decided.** The maintainer's GUI test found the screenshot's
background grid wrong: dark scratch lines over the part. The shot now
builds its own floor for the shot's fitted camera: the viewport's grid
law (`interact::nice_step` spacing, `GRID_MAJOR_EVERY` heavy lines, the
cubed rim fade) at a 40 px wanted spacing, with light inks on a light
ground, drawn through a new depth-tested `Canvas::ground_line` before
the parts so the part occludes the floor. The live camera's overlay grid
copy is dropped from the shot.

**Why.** `world::collect`'s overlay grid is spaced and centred for the
live camera; through the shot's fitted camera it landed at the wrong
scale and position, and `raster()` drew every overlay line with no
depth test, so it painted over the part. Pinning test:
`raster::tests::ground_lines_take_the_viewports_floor_law`, plus the
grid-ink assertions in `report_tests::the_screenshot_paints_the_document`.

**Considered and dropped.**
- Fading or deleting the grid: the floor helps the reader place the
  part; the viewport shows one, so the shot should show its own.
- Reusing the overlay lines with a depth test only: the spacing bug
  stayed - the lines were still the live camera's.
