## #2349 - Sinumerik refuses a G83 dwell

**Decided:** The Sinumerik post refuses a positive G83 (deep peck) dwell
by name, before it writes any line of that cycle. A G83 without a dwell
posts as before. The G82 dwell (P in milliseconds) is not changed.

**Why:** The post writes the ISO-dialect cycle form (`G83 X Y Z R Q F`),
not the native `CYCLE83`. Siemens' published *SINUMERIK 840D sl ISO
Milling* programming manual (section "Deep hole drilling cycle with chip
removal (G83)") gives the G83 format with X, Y, Z, R, Q, F and K, and
with no P word. Thus no public source verifies that the control reads a
P on this G83 as a bottom dwell, or in which unit. A word that is not
verified must not be posted (Prime Directive). This follows the refusal
that PR #2048 (#1820) uses for the Fanuc, Mazak and Doosan posts, with
the same message text.

**Considered and dropped:**
- Keep `P<ms>` on G83: not verified for this control; the control can
  ignore it or alarm, and the dwell at the machine then differs from the
  user's setting.
- Post the native `CYCLE83` with its dwell-at-depth parameter (DTB,
  seconds): this changes every G83 the post writes, mixes a native cycle
  into an ISO-form cycle group (G80, modal continuation), and needs its
  own goldens. It is a larger change than this defect.
- Drop the P word silently: this loses the requested dwell.

**Cost to change later:** To enable the dwell, cite a public Siemens
source for the word and its unit (or move to `CYCLE83`), remove the
refusal and change `sinumerik_refuses_g83_dwell`.

**Validation:** `crates/post/tests/sinumerik_g83_dwell.rs` pins the
refusal (0.001 s and 0.5 s), the unchanged G83 with no dwell, and the
G82 `P500`. No real control was used. The golden `synth_sinumerik.nc`
has no G83 dwell and does not change.
