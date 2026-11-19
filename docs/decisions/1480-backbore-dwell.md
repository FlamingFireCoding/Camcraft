## #1480 - Back bore (G89) carries the dwell and feeds out at the plunge feed

**What was decided.** The drill generator (`camcraft_motion::drilling::drill`)
now writes the user's dwell into the `Cycle` record of a Back bore hole. It
also feeds the tool out at the plunge feed, the same feed as the way in.
Before, the dwell was forced to 0 and the feed out used the cutting feed.

**Why.**
- G89 is "feed in, dwell, feed out" (RS-274/NGC, NIST IR 6556; LinuxCNC
  G-code reference). The cycle exists for its dwell, and the user could not
  get one.
- A canned G89 (and the G85 that several posts write for a Back bore with no
  dwell) has one F word. The post takes that F from the first cutting move
  of the hole's chain, which is the plunge. So the control always fed out at
  the plunge feed. The chain said the cutting feed. The verifier, the
  simulation time and the `.cl` dump then disagreed with the machine. The
  Ream cycle (G85, #1445) already uses the one-feed chain; Back bore now
  uses the same law.
- The dwell unit needs no new code. Each post already reads the dwell in
  seconds from the stream and writes its own P unit, or refuses by name:
  Fanuc, Mazak, Doosan, Haas and Sinumerik write whole milliseconds; grblHAL,
  Tormach, LinuxCNC and Mach4 write seconds; Heidenhain writes an explicit
  dwell block; the lathe posts and the posts with no canned cycle refuse the
  Back bore. `crates/post/tests/dwell_cycles_modal_p.rs`,
  `haas_dwell_cycles.rs`, `mach4_post.rs` and `heidenhain_cycles.rs` pin
  these words (#2350, #2360).

**Considered and dropped.**
- Keep the feed out at the cutting feed, and make each post write a second F
  word. A canned cycle has no second F word. A post would have to expand the
  cycle into G1 moves, which changes every Back bore program.
- A separate "retract feed" parameter for bore cycles. This is a new
  parameter and a new command field. It is not necessary for this issue.
- A new G89 golden for each post in this PR. The posts did not change, and
  their P words are pinned already. A Sinumerik G89 P pin is the one that is
  missing; see "Not exercised" in the PR.

**Cost to change later.** Small. The feed out is one line in
`crates/motion/src/drilling/cycle.rs`, pinned by
`back_bore_feeds_in_and_out_at_the_plunge_feed`. A Back bore program with no
dwell posts the same canned block as before on each canned-cycle post. The
`.cl` chain of a Back bore changes its feed-out feed. On the Heidenhain post,
which writes the Back bore as an explicit chain, the retract line now has
the plunge feed.
