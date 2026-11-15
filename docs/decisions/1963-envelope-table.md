## #1963 - Stand the displayed work envelope on the table

**Decided:** Translate only the viewport box Z range to `0..(max_z - min_z)`.
Keep the X/Y bounds relative to the setup work origin.
Keep lathe cylinders relative to the spindle and work origin.
Update the checkbox tooltip to state these references.

**Why:** The ground grid represents the table at world Z=0.
The display previously used the program-space travel range without this translation.
With `assets/machines/tormach-pcnc1100-s3.mch` and the fresh document implicit setup,
the bottom was Z=-419.1 mm. It must be Z=0 mm.
A shifted G54 origin and an inverted Z axis must also preserve the travel height.
X/Y have no equivalent table-height error: the grid defines a plane, not a table footprint.
The machine files do not supply a table footprint to justify a new X/Y placement.

**Safety result:** `crates/post/src/machine/envelope.rs` supplies display data only.
`crates/app/src/machine/mod.rs` checks travel directly against axis limits and program coordinates.
No travel, reach, collision, posting, or stock simulation check consumes the drawn box.
The fix does not change those checks or their coordinate reference.
The effort label stays `effort:medium`.

**Considered and dropped:** Changing machine travel limits or the shared program envelope
would change their meaning and could affect callers beyond the viewport.
Moving X/Y would require table placement data that the current files do not state.
Moving a lathe cylinder onto a mill table would lose its spindle reference.

**Cost to change later:** Replace the viewport translation when machine files provide
an explicit table transform. Update the value tests and tooltip with that transform.
No dependency, project format, or saved data changes are required.
