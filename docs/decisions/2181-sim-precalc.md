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

**Cost to change later.** The budget and the keyframe count are
constants in `crates/app/src/stock_play_precalc.rs`. Moving the start to
Generate needs only a different call site of `PlayWorker::spawn_precalc`.
