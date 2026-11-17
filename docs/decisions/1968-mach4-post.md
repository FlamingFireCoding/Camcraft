## #1968 - Clean-room Mach4 milling postprocessor (mach4.cps)

**What was decided.**
- Added `assets/posts/mach4.cps`, registered `"Mach4"` in `crates/app/src/posting.rs` (`BUILTINS`), and added Mach4 conformance entries and `.nc` goldens in `crates/app/tests/conformance.rs`.
- 3-axis programs use standard Mach4 Mill `G94` modal blocks (`G21`/`G20`, `G90 G94 G91.1 G40 G49 G17 G64`), `T.. M6` / `G43 H..`, `G54..G59` and `G54.1 P1..P248`, `G17`/`G18`/`G19` arcs, `G41`/`G42`/`G40` cutter compensation, `G73/G81/G82/G83/G84/G85/G86/G89` canned cycles with dwell `P` in seconds, and `G31` Macro B probing.
- 3+2 indexed AC trunnion sections (using `#1962`'s schema: `A` parent of `C`, `pivot`, `offset`, `zeroOffset`, `rollover`, `signConvention`, `controller.rtcp = false`) retract safely to home/clearance Z, optionally unclamp/clamp on separate blocks, position `G00 A.. C..`, and emit transformed `G94` XYZ coordinates and re-mapped cardinal arcs (`G17/G18/G19`) without `G43.4`.
- Refuses tilted moves by name when `controller.rtcp = true` (since Mach4 lacks `G43.4`), when required AC trunnion geometry/limits/orientation are invalid or unreachable, and refuses simultaneous 4/5-axis moves within a section by name containing `"needs #1966: inverse-time feed"`.
- Places G54 work datum at the C platter center at kinematic home (`work_datum = a_pivot + c_offset`, `wcs_offset = [0, 0, 0]`) so 3-axis G54 matches `A=0, C=0` while compensating `C.offset` during A tilts identically to `camcraft_post::kinematics::forward_part`.
- Defaults `safeRetract` to `"G28"` while supporting `"G53"` (requiring `safeRetractZ` on named machines) and `"G30"`, and enforces `rollover_360` `[0, 360)` rewrap only after safe Z retract.
- Stacked on PR `#2162` (`base-2162`, branch of PR target `--base main`) because `#1962`'s trunnion `.mch` schema lives in `#2162` and has not merged to `upstream/main` yet.

**Why.**
Mach4 Mill controllers do not support Fanuc `G43.4` RTCP, and #1966 (`G93` inverse-time feed) has not landed yet. Computing exact AC trunnion inverse kinematics and part-to-machine datum transformation in G94 makes both 3-axis and 3+2 indexed milling safe immediately while keeping simultaneous 5-axis gated cleanly until `#1966`. Putting the work datum at platter center preserves G54 continuity between 3-axis and 3+2 operations.

**What was considered and dropped.**
- Emitting simultaneous 4/5-axis feeds with `G94` or heuristic `G93` before `#1966` lands: dropped as unsafe feedrate math; refused explicitly instead.
- Defaulting `safeRetract` to `"G53"`: dropped because custom trunnion configs that omit `safe_retract` would fail unconditionally unless `safeRetract = "G28"` or explicit `safe_retract` is chosen; `"G53"` remains explicitly selectable and enforces `safeRetractZ`.
- Silently assuming `[0, 0, 0]` when `A.pivot` or `C.offset` is omitted from `.mch`: dropped so incomplete machine definitions are refused by name rather than crashing into trunnion arms.
- Leaving oblique 3+2 arcs as `G17` IJK: dropped; only cardinally aligned machine planes (`G17/G18/G19`) emit arcs (with CW/CCW sign flip if normal points along `-axis`) and oblique arcs are refused cleanly.

**Cost to change later.**
When `#1966` introduces `G93` inverse-time feed support, `preparePose` and `onLinear` in `assets/posts/mach4.cps` can branch on `G93` for simultaneous multi-axis feeds while leaving 3-axis and 3+2 `G94` paths untouched. If arbitrary non-platter work datums are later streamed explicitly, subtracting `workDatum - (aPivot + cOffset)` in `transformPt` is a one-line adjustment.

**PR #2296 review corrections.**
- Establish both rotary home commands after a safe retract before initial vertical work on a configured trunnion. An inherited controller angle is unknown, so relying on program-end state or retaining C at the vertical pole was rejected; repeated execution can otherwise cut at the previous run's final tilt.
- Route every probe hook through vertical pose preparation, including result/offset updates. This restores the untransformed probing coordinate frame after indexing and refuses unreachable home positions. Silently probing at the prior indexed angle was rejected.
- Declare the five existing Mach4 options with enum/string/integer types and their existing defaults. Keep programNumber as the fallback when the host supplies no program identity. Direct engine-only coverage was rejected because it bypasses CLI/app override validation; CLI acceptance and refusal tests exercise the declared schema.

**PR #2296 probe-approach review correction.**
- Seat the first probe X/Y traverse at clearance after a tool change or rotary reset, then use `G31` for the descent. The previous stream `StartZ` does not describe the retracted machine position. A rapid descent to that stale start was rejected as unsafe; refusing every indexed-to-probe sequence was rejected because a protected approach can preserve the supported workflow. Changing this later requires an explicit, tracked probe start position and renewed approach-motion regression coverage.
- Keep main's file-backed built-in registry in `posting_files.rs`, carrying Mach4 into that registry and its browser fallback. Restoring the old embedded registry was rejected because it would undo main's runtime asset loading. A later registry redesign must preserve both runtime loading and the browser fallback.
