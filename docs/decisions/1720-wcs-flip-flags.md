## #1720 - WCS axis flip: stored flags, refused at post and generation

**Decided (taken unattended, issue-worker-high):**

1. **Where the flags live.** A new component `schema::WcsFlip([bool; 3])`
   on the setup's WCS entity, next to its `Location`. Not on `SetupDef`:
   the flip belongs to the frame, and every place that moves, deletes or
   copies the WCS entity (DeleteEntity, DeleteSetup, CreateSetup's datum
   transfer) now carries it with the location. "No flip" is stored as no
   component, so a world without flips is unchanged.
2. **Project file.** An optional `flip` object on the entity row, with
   keys `x`, `y`, `z`; only a flipped axis is written (`{"x": true}`), and
   a row with no flip writes no key. A file without the key opens with no
   flip. A `flip` on a row that is not a WCS is refused as corrupt by
   name. `PROJECT_VERSION` goes from 66 to 67, because an older reader
   would silently drop the flip and post a program that does not match
   the stored frame once part 2 lands.
3. **Refusal, not output.** Part 1 changes no posted G-code. A flipped
   WCS is refused by name at the one generation door (`collect_job`, next
   to the #1510 tilt check) and in the post's verification gate (next to
   #315's rotated-offset clause), so every post path refuses it. The post
   clause gives one line per flipped setup, in first-use program order,
   with the operations that use it; suppressed operations are skipped,
   as the rotated-offset clause does.
4. **The command places a WCS when none exists.** `SetWcsFlip` on a setup
   whose WCS was never placed spawns it at the identity (what
   `SetWcsOrigin` would do), and its undo removes it again. Considered:
   refusing the flip until the origin is set; dropped, because the page
   shows the buttons on an unplaced WCS and a refusal there reads as a
   broken button.
5. **The datum transfer copies the flip.** When `CreateSetup` clones an
   existing setup's WCS into the new setup, the flip is cloned with the
   location (the new frame is the same frame until the user changes it);
   undo removes the clone. Materializing `Setup1` from a document with no
   setups keeps the WCS entity it points at, flip included. Considered:
   a clone that starts unflipped; dropped, because a copied frame that
   silently loses an axis direction is not the same frame.
6. **The gizmo shows the flip.** A flipped axis is drawn reversed, and a
   flipped WCS at the origin does not hide the world axes (it is not the
   world frame).
7. **No script verb.** The MCP tool `set_wcs_flip` covers the agent
   surface; the script language has no WCS verb family to join yet.

**Considered and dropped:** a signed scale per axis (`[f64; 3]` of +-1):
it admits values that mean nothing; three booleans do not.

**Cost to change later:** the key shape is in v67 files; renaming it needs
a migration step. Part 2 (posting with a flip) replaces the two refusals
with the frame work and keeps the stored flags.
