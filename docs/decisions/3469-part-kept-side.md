## #3469 - Part-off: the user states the piece kept and a back stock

**Decided.** The Part (part-off) operation gets one setting,
`kerf: PartKerf` (`motion::turning::parting_law`), with two values:

- `kept`: which piece is the finished part. `CutOff` (the default) is the
  #3452 rule: the piece cut off the free end is the part, and the whole
  kerf is on the chuck side. `Chuck` keeps the piece in the chuck: the
  whole kerf is on the free-end side, and a stub comes off the free end.
- `back_stock` (mm, 0 or more, default 0): the whole kerf stands this far
  off the parting plane, into the scrap side. The part keeps that much on
  its end face for a later facing pass.

The page shows them on the Part op's Passes tab as "The part is" (Cut-off
piece / Chuck piece) and "Back stock". The project file has the keys
`kept` (`cut_off` / `chuck`) and `back_stock`. Each key is written only
when it is not the default. MCP `edit_operation` patches them through
`EditOperationParams` (one undo step). A negative or non-finite back
stock is refused at the command door (`is_sane`), in the kernel and in
the verdict.

**The gouge law judges against the kept part.** `part_kerf_refusal_for`
reads the scrap side from `kept`, and it treats the back stock as part
material. The back stock zone has no model facets, so its radius is the
model's largest radius within one back-stock length of the end face. A
blade band in that zone, below that radius, is refused by name.

**The flipped-setup refusal of #3472 is narrowed, not removed.** In a
setup that holds the part end for end, the default (`CutOff`) is still
refused, and the refusal now names the setting: "Set 'The part is' to
'Chuck piece'". With `Chuck` the part-off generates.

**Existing projects post the same program.** The default is the #3452
kerf. The stamp folds the setting only when it is not the default, so no
op saved before the setting reads stale. No golden or catalog file moves.

**Why.**
- The maintainer's scope (issue comments of 2026-11-22): one parameter
  with a default, "the piece that is cut off (default) / the piece that
  stays in the chuck", the kerf fully on the other side, and the back
  allowance kept in scope.
- One struct field instead of two keeps each `OpParams::Part` literal a
  one-field change, and the kernel and the verdict read one value.
- The flipped-setup default stays refused because the default there cuts
  into the finished part. The user has a way out: the setting.

**Dropped.**
- A "blade centred on the plane" option (the first text of the issue).
  It puts half the blade into the part, which the gouge law must refuse.
  A part that a later op faces to length is a back stock instead.
- Removing the flipped-setup refusal entirely. The default in such a setup
  would cut the finished part with no warning.
- A setup or machine field for the chuck side. The issue scope puts the
  side on the operation. A setup field can come later and set the default.
- Folding the setting into the stamp always. That would read every saved
  Part op stale once with no change to its program.

**Cost to change later.** The file keys are additive. A third `kept`
value, or a setup-level default, adds an enum arm and a DTO arm.
