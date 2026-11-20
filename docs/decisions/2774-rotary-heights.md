# #2774 - the rotary kinds' heights are radii about the fitted axis

## #2774 - rotary heights read as radii about the fitted rotary axis

**What was decided.** A rotary operation can read a height as a RADIUS about
the fitted rotation axis, not only as a world-Z plane. The stored form is
`RadialHeight { source, offset }` with `RadialSource` = stock OD, model max
radius, or absolute radius. The fields are optional: `clearance_r`,
`retract_r`, `top_r`, `bottom_r` on Rotary Contour and Rotary Parallel, and
`clearance_r` + `top_r` on Rotary Offset. `None` keeps today's Z-plane law
byte-for-byte; the old `*_z` fields stay. One resolver
(`app::operation::radial_heights`) fits the SAME axis the generators pose
about, and the gate (`radial_gate`) refuses at Create, Edit and ChangeKind
when a clearance or retract radius stands inside the work, or when a stated
band crosses (top below bottom). Refusals name the field and the radius.

**Why.** On a turned bar the tool must keep its distance from the axis all
the way round; a Z plane says nothing there. Keeping the fields optional
means an old project file, and every rotary op that never sets a radius,
posts exactly what it posted before. The refusal at the command follows
"Validate early" (Prime Directive): the generator's own law is stated at
the command, so no silent gouge rapid can reach the post.

**Seeds - corrected during the audit.** New contour and parallel ops seed
`clearance_r = stock OD + 5.0` and `retract_r = stock OD + 10.0`; the band
radii stay `None`. Rotary Offset seeds `clearance_r = stock OD + 5.0` with
`top_r: None`. The WIP commit seeded the band radii from stated numbers
too; that changed roughing output on stock out of the box (a regression we
could not pin as wanted), so the seeds drop the band. The panel's unchecked
rows let a user state a band radius on purpose.

**Project version 74 - migration is a comment-only no-op.** The new fields
are `Option` and serde-defaulted: a v73 file reads with all radii `None`,
which is exactly the old behaviour. One consequence: an `OpParams` change
moves the operation stamp, so an old project's cached toolpaths read STALE
after the upgrade and regenerate on the next generate. No data is lost; the
user sees a regenerate, not a wrong program. The current-format example
(`examples/first-continuous-4axis-part.camproj` / `.nc`) was advanced to
v74 in a second, test-only commit: the committed `.nc` was already stale on
main (a regen on main itself moves the same seven trailing A-word digits),
and the branch regen is byte-identical to the main regen. The `.camproj`
diff is the version number and the stamp.

**Viewport law - rings are ADDITIVE.** The radial rings the Heights tab
states draw into the viewport as extra rings about the fitted axis; they
never replace the Z-plane rectangles. The existing `law_tests.rs` viewport
assertions needed zero changes.

**Panel row scoping (review fix on this branch).** The four radial slots
share one row helper, and its fields share one id salt. Two enabled slots
collapsed into one egui id: the tree showed ONE From combo carrying the
last drawn row's selection. `radial_row` now scopes its fields with
`ui.push_id(title)` - the same law `height_editor` follows with its tag.
The new structural test `crates/ui/src/egui_app/radial_rows_tests.rs`
pins it: one combo per enabled slot, one echo per enabled slot, and the
checked/unchecked state per slot.

**MCP surface.** No new command was added: the radial fields are new
optional members of the existing operation parameters, and the MCP
create/edit tools round-trip them through the same params JSON (serde
defaults keep old scripts working). If a script verb turns out to need a
dedicated radial entry, that is a `P5` `source:agent` follow-up, not a
blocker for this PR.

**Considered and dropped.**
- Required radial fields (breaking): dropped - it would force a migration
  and re-gate every rotary program for no safety gain.
- Replacing the Z planes when a radius is set: dropped - the gate's band
  law and the generators need both readings (Z still orders stations along
  the axis), and the additive viewport law keeps the two pictures honest.
- A second axis fit in the UI echo: dropped - the echo reads the app
  resolver through the `height_memo` cache, so the panel, the rings, the
  gate and the generator answer for one and the same line (frame-cost law,
  #2238; counter test in `crates/ui/src/height_memo_tests.rs`).

**File-size note.** `crates/ui/src/operations.rs` stands at 654 lines; it
was 597 on main. The 400-line law asks for a split along the domain seam;
the echo helpers (the twin pair `height_echo` / `radial_echo` and
`resolve_all`) are the natural seam. The split is deferred to keep this
PR's diff to the feature; it owes a small refactor follow-up.

**Test-environment note.** The `camcraft-ui` lib suite in parallel threads
crashes one run in two on this machine (STATUS_ACCESS_VIOLATION). It does
the same on origin/main, so it is pre-existing and not this branch. The
full lib suite was verified with `--test-threads=1`: 1280 passed, 0 failed,
7 ignored (main serial: 1277 + 1 earlier radial cache test + 2 new radial
row tests).

**Cost to change later.** The stored form is the expression, so moving the
default source set or the seed offsets is a one-line change in
`kinds::seed`. Making the band radii required, or changing what
"stands inside" floors on, touches the gate and its pinning tests in
`crates/app/tests/rotary_verify.rs` only.
