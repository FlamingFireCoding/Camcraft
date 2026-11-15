## #1800 - A wire taper the machine cannot post is refused at the command door

**What was decided.** The Wire Contour create, paste and edit commands
(`wire_gate` in `crates/app/src/commands/operation/gates.rs`) now call
`app::machine::wire_edm::taper_door`. A non-zero `taper_deg` is refused by
name when the mounted machine states no `edm.max_taper_deg`, when the taper
is past that maximum, or, on a U/V machine (no `edm.taper_angle`), when the
machine states no guide planes (`edm.lower_guide_z`, `edm.upper_guide_z`).
The GUI page and MCP `edit_operation` both pass this door. A zero taper is
always admitted. An angle-word machine needs no guide planes for the door.

**Why.** Before, a taper the machine cannot post was accepted as a
parameter and refused later: the maximum at generate, the guide planes at
the post. A parameter that looks configurable and then always refuses is
misleading (the issue). The safety floor forbids the other branch of the
issue's "Done means": no shipped `.mch` gets an invented `max_taper_deg` or
guide height (the unstated-limits law of `assets/machines/AGENTS.md`). The
stream half is already pinned on main: `wire_taper.rs::
a_tapered_punch_posts_its_golden_and_its_cl_shows_the_poses` posts a 3
degree taper on the synthetic fixture to `wire_taper_punch.nc`.

**Considered and dropped.** Putting the check in `wire_op::condition_of`
(it also runs at generate): that file is already over 400 lines, and the
generate and post refusals stay as defence in depth for a machine changed
after the op was made. Gating 4-axis cuts the same way: out of scope for
this issue; their generate refusal (motion `ruled.rs`) stays.

**Cost to change later.** One function and one call. The existing tests
moved two assertions: the "past the maximum" error no longer carries the op
name (the command error does not name it), and the "no guide planes" case
moved from the post test to the new door test.
