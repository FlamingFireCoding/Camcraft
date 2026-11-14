## #1702 - batch runs the kernel suites; gate gets --only and --failed

**Decided:** `cargo xtask gate batch` runs the debug suites of `camcraft-kernel`
(`kernel` is now in `BATCH_CRATES`, after `geometry`). Every gate tier but
`test` takes `--only <step>` (repeatable) and `--failed <gate-logs/run>`. A
step matches `--only` when its name equals the word or starts with it and a
non-word character follows (`kernel` selects `kernel suites`). `--failed` reads
the `NN-<step>.log` files of an earlier run. A log with a `test result: FAILED`
line or a line that starts with `error:` marks a failed step. Both options
together keep only the steps that both name. An empty selection is an error
that lists the step names. `--dry-run` prints the selected steps.

**Why:** the release gate of the v0.20.0 cut failed in the kernel step, which
no earlier gate ran. A fix for one step cost a full run of about 90 minutes.
The maintainer, 2026-11-14: "catch kernel lanes in the xtask gate.. so we dont
have to start over 3 times".

**Considered and dropped:**
- A `summary.txt` written by each run for `--failed`: it would not exist in
  the logs of runs made before this change, such as the failed cut run.
- Selecting by step number: numbers change when the plan changes.
- Moving kernel out of the release tier: release still runs every crate.
  `SLOW_BINARIES` is unchanged (no kernel entry; kernel has no suite over 40 s
  in the list), and `batch` still skips its four slow binaries.

**Cost to change later:** the batch tier grows by about 3 minutes (the
kernel debug suites). Drop `kernel` from `BATCH_CRATES` to undo it.
