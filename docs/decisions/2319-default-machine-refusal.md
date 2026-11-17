## #2319 - A missing or malformed default machine file refuses the post

**What was decided.** The post paths of the app read the default machine
through `posting::default_machine_checked()`, a `Result`. When a mount has no
machine and the default machine file (`assets/machines/3axis-vmc.mch`) does
not read, is not JSON, is not a JSON object, or is an object with no
`name` or no `kinematics.static` object, the post is refused with a message
that names the file. Those two keys are what the post reads for the program
header and the coolant words. The shipped `3axis-vmc.mch` has both. It is
the only file that is read as the default machine, so no shipped file is
refused. No program is written. The three post paths
that call the engine use the new `posting_mounts::posted_machine_for`: the
single-file post, the per-mount post and the per-operation post.

**Why.** Before this change, `default_machine()` gave `Value::Null` for a bad
file. The engine reads `Value::Null` as "no machine". The program was then
written with zero machine limits and an empty coolant list. The test
`default_machine_missing` proved it with a real CLI process: exit 0 and a
complete Fanuc program, for both a missing and a malformed file. A manual
diff of `examples/first-part.camproj` against a run with the good file shows
the damage: the header changes from `O0001 (VMC 3-AXIS)` to
`O0001 (CAMCRAFT)`, and all three `M8` and all three `M9` lines are gone.
The motion is the same, so the program looks correct but runs dry.

**A refusal is not kept.** Only a machine that passed the check is kept for
the life of the process (`posting::KeptMachine`). The file text is not kept
before the check. A failed read or a refused file is not kept: the next post
reads the file again. A user who repairs the file can post again without a
restart, so the text "reinstall the file" stays true.

**Where the refusal is.** In the app, where it resolves the default machine,
not in the engine. `engine::run` with `Value::Null` stays a legal input: the
`post` binary of `camcraft-post`, the script check in `commands/post.rs`
(`engine::run(&text, &Value::Null, "")`) and many tests use it on purpose. A
refusal in the engine would change that API for all of them.

**Considered and dropped.**
- Change `machine_value_for` / `machine_value` to return `Result`. About 50
  test call sites and the sim traverse clock read them as a `Value`. The sim
  clock only times rapids, and it does not write G-code. `default_machine()`
  keeps its `Value` form for those readers, and its doc says the post never
  reads it.
- Parse the default file with the full `.mch` parser and mirror it, as a
  mounted machine is. That can change the posted bytes of every document
  without a machine (golden files). It is not needed for this defect.
- A full schema check of the default file. An object with a name and a
  `kinematics.static` section but no limits still posts with the engine's
  defaults. The limits of the default machine are not judged for a document
  with no mount: that is an older gap, tracked in #2321.
- Add "and restart" to the text, with the refusal kept until restart.
  Not keeping a failure is cheaper for the user.

**Cost to change later.** Small: one function (`posted_machine_for`) is the
one door from a mount to the engine's machine value. The parse and the
check are in `posting_default_machine.rs`, not in `posting.rs` (#2322 tracks
the split of `posting.rs`).
