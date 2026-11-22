## #3452 - Part-off: the whole kerf stands on the waste side of the part's end face

**Decided.** The Part (part-off) operation's "Bottom Z" (the parting plane)
is the finished part's END FACE. The whole kerf of the blade stands on the
waste side of that plane, one flank on the plane. The blade's CL (the middle
of its tip edge) is half a blade width from the plane, on the waste side.
The posted corner (#641) is unchanged: orientation 2 posts the -Z corner,
orientation 1 the +Z corner, orientation 6/9 the centre.

The issue's case (end face Z-70, 3 mm blade, orientation 2, bar to Z-84):

| | CL | kerf | posted Z (orientation 2) | part length |
|---|---|---|---|---|
| before | Z-70 | Z-71.5..-68.5 | Z-71.5 | 68.5 mm, chamfer lost |
| after | Z-71.5 | Z-73..-70 | Z-73 | 70 mm, chamfer kept |

With orientation 1 the same kerf posts Z-70 (its +Z flank, on the face).

**The waste side is derived, not assumed** (`motion::turning::parting_law::waste_side`):
- a plane at the model's low end (within the tolerance): the scrap is below;
- a plane at the model's high end: the scrap is above (the mirrored setup,
  the bar on the +Z side);
- a plane that the model runs on past on both sides (a model that carries
  its own bar): the side where the setup stock runs on further past the
  model; with no stock to read, the -Z side (ISO 841 lathe Z: +Z points away
  from the workholding, so the chuck is at -Z).

To read the bar's near end, `StockEnv` gets `s_bot` (the lowest station of
the stock in the part's frame), beside the existing `s_top`.

**The refusal "nothing to part"** now tests the kerf against the BAR (the
model's span widened by the stock's two ends), not against the model only.
The correct kerf beside the part's end is accepted when the stock runs on.
A kerf that meets no bar is refused with its numbers, for example
"nothing to part - the 3.000 mm kerf beside the parting plane (Z-73.000 to
Z-70.000) does not meet the bar (Z-70.000 to Z0.000); set the setup's stock
if the bar runs on past the part". With no stock set and the plane at the
model's end, there is no known material to part, so this refuses (before,
the centred kerf cut 1.5 mm of the part and passed).

**The generation verdict** (`part_kerf_refusal`, called in the Part arm of
the verdict at generation) refuses a part-off path whose cutting move puts
the blade's band (its own width, never wider) into model material on the
part side of the plane by more than 1 µm. It is a refusal (`GenError`), not
a warning: the post never sees the path. The meridian scan
(`blade_warnings_excess`) still reads a through-axis kerf as scrap; that is
correct for a kerf on the waste side, and the new law covers the part side
that the scan cannot see. Groove Rough and Groove Finish do not run the new
law: their kerf is inside the groove window by construction (their centres
step from the window edges, `blade.rs` header), so no legal groove can be
refused by it.

**Dropped.**
- Keep the blade centred on the plane and move the plane by half a blade in
  the generator only: the refusal and the verdict would still judge the
  wrong band.
- Assume the scrap is always at -Z: wrong for a setup whose bar runs on at
  +Z (the issue asked for the side from the setup).
- A warning in place of a refusal: a gouge of the finished part must block
  the post (Prime Directive).
- A user parameter for the kerf position (a facing allowance on the back,
  the blade centred for a later second-side facing): a strategy choice is a
  parameter with a default, but it is new behaviour, not this P1 fix. It is
  filed as a follow-up issue.

**Cost to change later.** Posted output of every existing part-off moves by
half a blade width toward the waste side (breaking post-output change).
A later "kerf position" parameter defaults to this behaviour, so it changes
no program.
