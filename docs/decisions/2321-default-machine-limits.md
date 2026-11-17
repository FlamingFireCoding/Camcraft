## #2321 - A document with no mounted machine is judged against the default machine's limits

**What was decided.** The program gate (`posting_mounts::machine_problems`)
now also judges the ops of each mount with no machine. It judges them
against the limits that the default machine file (`3axis-vmc.mch`) states:
the spindle range and the feed rate of each linear axis. These are the
same two judgements that a mounted machine's check makes, in the same
words. The refusal names the default machine, each operation and each
limit, and says what to do: mount a machine on the setup, or change the
operation. The status line's machine vote (`machine_checks`) is not
changed: it still says nothing when no machine is mounted.

**Why.** The post writes a document with no mount with the default
machine's values (#2319), so the gate must judge the program against the
same values. Readings on the code before the fix:
- An op at 7000 mm/min (above the default X/Y rate of 5000) posted with
  no refusal (`no_mounted_machine_is_refused_above_the_default_feed_rate`).
- An op at 15000 rpm (above the default 12000) was refused only by the
  engine's warning `spindle rpm over machine maximum`. That text names no
  operation and no limit. The gate now refuses it first, by name.

**How the default is read.** The default file is written in the post
grammar (`kinematics.static`), not in the `.mch` file form, so `parse_mch`
reads it as an empty machine. `posting::default_limits` reads the name,
`head.spindle.min/max` and each X/Y/Z axis's `speeds.normal` (feed) and
`speeds.rapid`. A zero, negative or absent number is unstated and checks
nothing (the unstated-limit law). The value is the one that
`default_machine_checked` gives to the engine, so the gate and the post
read the same numbers. A default file that does not read is refused in
the gate too, by its name (#2319).

**Considered and dropped.**
- Give the default machine to the full `machine::check_ops`. That check
  also judges the pose: a tilted or C-axis move on a machine with no
  rotaries is refused. The default file states no rotaries, but the
  issue asks for its limits, not for a new pose law on every document
  with no machine. That change is larger and is not taken here.
- Say only in the program report that no limits were checked. The issue
  permits it, but a refusal that names the limit is safer (Prime
  Directive), and the post already reads these values.
- Change the status line vote to show the default machine. That is UI
  behaviour that the issue does not ask for; the post gate is the
  safety rule.
- Put the code in `posting.rs`. It is 1535 lines (#2322); the reader and
  the check are in `posting_default_machine.rs`, the clause in
  `posting_mounts.rs`.

**Cost to change later.** Small: one clause (`default_machine_problems`)
and one reader (`default_limits`).
