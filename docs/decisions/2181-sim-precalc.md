## #2181 - The stock removal of playback is calculated before playback

**Decided.** When the transport starts the traced playback, the
playback worker first cuts the whole program once. It keeps keyframes of
the cut stock (`FieldPlay` checkpoints) at a fixed stride. After that,
each playhead position is shown from the keyframe at or before it, plus
at most one stride of cutting. A jump forward or back never cuts the
program again.

- **Where:** in the existing playback worker (`stock_play_worker`), at
  the first Play of a document version. No new thread, no new cache.
- **Stamp:** `State::doc_version`. Each command increments it, so an
  edit of an operation, a tool, the stock, a fixture or a setup retires
  the worker and its keyframes (the worker is dropped and its thread is
  joined). An operation delete is a command too: the keyframes are freed
  with the worker. A stale calculation cannot be shown.
- **Memory:** a budget of 512 MiB of keyframe cut layers
  (`PRECALC_BUDGET`), at most 1024 keyframes (`MAX_KEYFRAMES`), at least
  64 moves between keyframes (`MIN_STRIDE`). Past the budget the store
  thins: every second keyframe goes and the stride doubles. At worst only
  the uncut stock stays and playback cuts live, as before.
- **Cancel and failure:** the transport shows the progress and a Cancel
  button. A cancel, a stop or a refused cut gives `Precalc::Incomplete`
  with the reason, and the line says "incomplete". The keyframes made
  before the stop are exact prefixes of the program, so they stay. The
  moves after the last keyframe are cut live.
- **Display only:** the keyframes feed only the playback picture. The
  Deviate verdict and the post gate (`posting::verification_gate`) are
  calculated elsewhere and do not change. A test pins the gate result
  before and after a calculation.

**Why.** The field engine cuts order-free: a prefix of the program is
cut exactly by its own moves. A keyframe plus the remaining moves is
therefore bit-identical to the live cut (pinned at every position on a
small face). The checkpoint store and its backward restore existed; the
change adds the calculation pass, a forward restore, a byte budget and
the state readout. This is the smallest change that removes the lag of
a long jump without a second simulation path.

**Dropped.**
- One stored state for each move: memory grows with the program; a
  dense face has hundreds of thousands of moves.
- Copy-on-write sharing of field bricks between keyframes (denser
  keyframes for the same memory): it changes the simulation crate's
  storage and is a separate slice.
- A calculation directly after Generate: it costs time for documents
  that are never played, and each edit would start it again.
- Frames shown while the calculation runs: the worker serves no request
  until the calculation ends (the sheet stands, the line shows the
  progress). Serving frames during the calculation needs a second field.
- Pausing the transport until the calculation ends: the transport keeps
  its own clock; the picture follows when the calculation is done.

**Review fix (#2192).** The keyframe store and the calculation loop
moved to the simulation crate:

- `camcraft_sim::play_keys`: `Keyframes`, `Precalc`, `PRECALC_BUDGET`,
  `MAX_KEYFRAMES`. Each keyframe goes in through `Keyframes::offer`. The
  calculation and the live seek use the same path, so the byte budget
  also holds when live playback seeks after a cancel.
- `camcraft_sim::play_precalc`: the loop `drive`, with the
  `PrecalcCut` and `PrecalcOwner` seams.
- `app` keeps only the methods of its own types (`FieldPlay`,
  `PlayWorker`) and re-exports the sim items from `stock_play`.
- No new crate edge: `camcraft-app` already depends on `camcraft-sim`.
- Dropped: a move of `FieldPlay` and `PlayWorker` to the simulation
  crate in this PR. They existed before #2181 and their move touches
  the playback suites of other issues. Issue #2268 tracks it.

**Second review fix (#2192, finding 4 again).** The rest of the new
domain logic moved to the simulation crate as well:

- `camcraft_sim::play_precalc::{begin, step, Calc, Playback}`: the
  stride choice, the store restart and the offer of each boundary.
  `FieldPlay` implements `Playback` with bindings only (its total, its
  playhead, its store, a rewind and a cut). `begin_precalc` and
  `precalc_step` stay as one-line calls for the app tests.
- `camcraft_sim::play_precalc_mailbox::{Mailbox, MailboxOwner}`: the
  progress slot, the cancel flag and the "lost state reads incomplete"
  law. The worker binds its stop flag and its clock as closures. The
  wakeup, the thread and the frame publication stay in app.
- `Keyframes::restores`: one seek law ("restore when behind, or when a
  keyframe is more than one stride ahead") for `FieldPlay::seek` and
  the worker's step choice. Before, the two places had their own copy.
- Chosen: closures for the stop and the clock, not a new trait. The
  owner has only two bindings, and a test passes them in directly.
- Dropped: a move of the `Shared` struct of the worker. It also holds
  the request and the frame slots of the playback before #2181; #2268
  moves it with `PlayWorker`.
- The behaviour does not change. The `stock_play_precalc` suite pins
  the bit-identical cut, the replay law, the budget, the cancel and the
  equal post gate result before and after.

**Cost to change later.** The budget and the keyframe count are
constants in `crates/sim/src/play_keys.rs`. Moving the start to
Generate needs only a different call site of `PlayWorker::spawn_precalc`.
