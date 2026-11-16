## #2265 - Each solid pass start of a pocket gets the asked entry

**Decided.** In the pocket family (Pocket, rest 2D, Adaptive), a pass start
that is in material (`plunge_law::into_solid`) is entered by the entry the
user asked for (helix, ramp or arc), not by a straight plunge. This covers
each separate piece of the area at each level, on the HSM plan and on the
Offset rings fallback. A helix is used only where it closes on the pass
start or links to it at depth. Else the tool ramps along the pass. Else
(a pass too short to ramp) it plunges, and a note names the site, the
level Z and the reason. The HSM plan gives each piece after the first its
own entry point from `choose_entry`.

**Why.** The maintainer's report: the first region got the helix, the
other regions got a straight plunge into material with no warning. That
can break a tool (Prime Directive). The root cause was the pass link,
which only gave a helix or ramp to tools that are not centre-cutting.

**Kept.** A plunge into a column that is already cleared, or into air, is
unchanged. An operation whose entry is Plunge is unchanged. The rule for
tools that are not centre-cutting (`restart`, refusal by name) is
unchanged. No verify check, post gate or refusal changed.

**Dropped.**
- Turning the rule on for every planar ladder (contour, slot, trace). Those
  passes start on the part wall or at a slot end, where the existing entry
  laws apply; a change there needs its own goldens and review. Follow-up #2269.
- A warning only, with the plunge kept. The user asked for a helix; a named
  plunge is the last resort, not the default.

**Cost to change.** One flag (`Motion::with_solid_restart`) in
`pocket_emit.rs` opts a ladder in. Removing it restores the old pass link.
