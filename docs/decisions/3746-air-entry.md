## #3746 - 2D Offset "Pass starts at": the Arc entry starts in air outside the stock box

**Decided.** On a start-side ring plan round a part (2D Offset, "Pass
starts at" Inside or Outside, or Auto once `ring_side::resolve` picks a
side), the Arc entry of each level is planned by `air_entry`:

1. A plunge in air at a point P. The distance from P to the stock box (the
   bounds of the stock footprint) is at least the tool radius plus the
   approach distance (#3745).
2. In the cut plane: a straight lead from P where it is needed, then one
   G17 arc of Arc radius and Arc sweep (Arc radius 0: the tool diameter;
   Arc sweep 0: a quarter turn), tangent to the first cut at the pass
   start. The straight lead is tangent to the arc.
3. The lead is proved with the proof of the horizontal arc entry
   (`descent_xy::arc_at`: the inscribed chord fan and the circumscribed
   hull), against the cutter-centre area joined with the air outside the
   stock box, and across none of the planned passes of the level. So it
   touches no part and crosses no ring (the verifier's Crossing law holds
   the post for a feed that crosses another feed at one Z).
4. A lead whose arc starts with the tool disc still in air is taken first:
   then the straight lead cuts nothing and the arc is the first cut. The
   arc radius is the user's; it is halved only where no side of the first
   cut has room, and the path then names the radius that it flew.

**Where no air entry fits** (the Arc entry is not changed silently): the
entry stands down, in this order, to a helix and to the ramp along the
first cut. Each stand-in is named
(`entry: Inside pass - no air entry outside the stock box fits (...) -
... instead`). With none of them the operation is refused by name
(`AIR_ENTRY_REFUSAL`). The axial dive into material that the Arc entry
gave before ("the arc's lead room is blocked - axial dive on the floor
instead") is no longer a stand-in on a start-side pass. Dropped: the
vertical arc ramp as a stand-in. On the dog bone its first chords come
down within 30 degrees of the tool axis into the stock, and the plunge law
(`plunge_law::first_solid_plunge`) reads them as a plunge into material.

**Inside.** The first Inside ring runs along the part wall, and the other
rings of the level enclose it. Each in-plane lead from the air outside the
box must cross those rings at the cut Z. That is a crossing of two feeds at
one depth, which the verifier's Crossing law reads as a defect and which
holds the post. Thus the recommended option of the issue (a lead that
feeds through the stock to the first ring) is dropped: it would refuse
every Inside post. Inside takes the air entry only where a lead reaches its
first ring without a crossing (for example an open piece that the stock
box cuts). Else it takes the named stand-in above (a helix, then the
ramp). It is never a plunge into material. A change
of the Inside order (cut the outer rings first) would change the
maintainer's "Inside" and is not taken here.

**Scope kept small.**
- Only the Arc entry changes. A user-selected Helix, Ramp or Plunge keeps
  its entry. The warning for that choice and the Arc default are #3748.
- Only the first entry of a level changes. The re-entries after a lift
  (`entry_restart`, the pass links) are #3747 and #3748. The "plunged into
  material" notes come from those re-entries.
- The avoid route (`pocket_avoid`, fixtures and clamps) keeps its old
  entry: its keep-outs are not known outside the area, so the air there is
  not proved. Follow-up: #3810.
- The Lead in / Lead out page on `main` offers all four entry kinds for
  each start side and greys none out (`lead_page.rs`), as the maintainer's
  clarification on #3748 asks. No page change here; the warning text next
  to the selector is #3748.

**Cost to change later.** The rule lives in `air_entry.rs`. #3750 moves it
into the shared `ring_entry` module.
