## #3316 - "Pass starts at" is one shared parameter (2D Offset first)

**What was decided.**

- One domain type, `machining2d::PassStart { Auto, Inside, Outside }`,
  in `crates/motion/src/machining2d/pass_start.rs`. `app` re-exports it.
  Every kind with a pass order uses this type. No kind gets its own enum.
- One project key, `pass_start` (`project::PassStartDto`, snake case).
  `auto` is the default and is never written. Files without the key read
  as Auto. There is no version step, and every file without a choice
  stays byte-identical.
- One command: the field is a plain `OpParams` field, so
  `EditOperationParams` carries it (undo and redo included). One MCP
  tool: the generic `edit_operation` merge patch. The patch guard keeps an
  explicit `"auto"` (`pass_start_kept` in `mcp/edit.rs`), because a
  skipped default never comes back from the round-trip.
- One Passes-tab row, `Pass starts at`
  (`crates/ui/src/egui_app/pass_start_row.rs`). A kind shows the row when
  its pane passes `Some(&mut pass_start)`.
- The digest writes the value only off Auto, so every earlier operation
  keeps its cache stamp.
- The ring law (`PassStart::rank`), read by `waterline_passes`:
  - Auto: the core first (`kmax - k`). This is the same stable sort as
    the old `Reverse(k)`, so Auto is byte-identical.
  - Outside: the loops of the outer boundary family (counter-clockwise)
    from the boundary in, then the hole families (clockwise) from the core
    back to their wall. On an Outside containment band the outer boundary
    is the stock reach and the hole is the part, so the level starts with
    the tool outside the stock and ends on the part wall. Each ring has
    cut air on one side: no ring is a full slot.
  - Inside: the mirror (holes from the wall out, then the outer family
    from the core back).
- Only 2D Offset reads it in this PR. The motion entry points are
  `pocket_started` and `pocket_avoid_started`; the old
  `pocket_entered_with_relief` and `pocket_avoid_with_relief` call them
  with Auto. `plan_passes` calls `plan_passes_from` with Auto, so the 3-D
  pocket, the HSM fallback and rest machining are not changed.

**Room for the #1448 start point.** The start point (where on its ring a
pass starts) and the exit point go beside `pass_start` as their own
fields with their own key. The planner input is the pair: the ring rank
chooses the order, the start point rotates each loop to its entry. The
same entry functions take the new input when #1448 lands.

**Why.** The maintainer asked for one parameter with one shape for every
kind (#3316 comments, 2026-11-22). The field rides the existing command
and MCP door, so the GUI and agents use the same undoable path.

**Considered and dropped.**

- A `pass_start` field on `PocketParams`. About 57 struct literals
  without `..` would change for one value that only one kind reads now.
  An argument on the new entry functions keeps every other caller
  byte-identical. When Pocket and Adaptive take the parameter (their
  sub-issues), the field can move into `PocketParams` in one step.
- A change of Auto for an Outside band. The maintainer said Auto keeps
  the current behaviour.
- A pure distance-from-boundary rank. It interleaves the two families
  where the band width changes, and then a ring can have uncut stock on
  both sides.

**Cost to change later.** Low. The type and the key are shared already.
A new kind adds its field, its row call and its planner read.
