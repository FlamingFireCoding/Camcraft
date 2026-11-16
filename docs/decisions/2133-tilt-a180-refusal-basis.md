## #2133 - Out-of-range 3+2 tilt refusal and A0 B180 plane X along machine X

- **What was decided:**
  1. Keep `Tilt`'s canonical letter bounds (`A` in `-90..=90`, `B` in `-180..=180`) and add `Tilt::refusal` in `crates/app/src/operation/types/params.rs`, wired into `OpParams::named_refusal` (`crates/app/src/operation/types/sane.rs`) and `feedback::apply_hint` (`crates/ui/src/feedback.rs`). When `Tool axis A` or `Tool axis B` is out of range, the refusal names the field, states the limits (`-90..90` and `-180..180`), and explains that the face opposite the top uses `Tool axis A 0` and `Tool axis B 180`. Non-finite angles and out-of-range plane origins are also refused by name.
  2. Update `TiltedFrame::new` (`crates/cl/src/cl/frame.rs`) to zero out axis components smaller than `1e-12` when selecting the tie-breaking seed axis (`seed_idx`). Thus `A 0 B 180` (where `sin(180 deg) ~ 1.22e-16` previously lost to `0.0` in Y) picks machine `+X` as plane X, matching `A -90`, `A 0` and `A 90`.
  3. Stack on PR #2135 (`issue/1904-first-4axis-part`) and update `site/content/guide/getting-started/first-4axis-part.md`, `crates/app/tests/first_4axis_part.rs`, and `examples/first-4axis-part.camproj` so `Flat A180` uses the same `-0.75, 0` to `-0.25, 0` ends as the other three flats (`examples/first-4axis-part.nc` is byte-identical).

- **Why:**
  - Refusing by name preserves the canonical-pose invariant (`Tilt` is stored and stamped as authored, never silently rewritten at command creation) and keeps existing command-door tests (`an_insane_tilt_refuses_at_the_command_door`) and `indexed.md` accurate, while surfacing the remedy in GUI Apply hints, GUI refusal banners, project validation, and MCP errors.
  - Suppressing trigonometric dust (`< 1e-12`) in `TiltedFrame::new`'s seed tie-breaker fixes the root cause of the `A 0 B 180` axis swap without altering `angles_to_axis`, `axis_to_angles`, or any non-coordinate-plane tilt frame.

- **What was considered and dropped:**
  - **Accepting `A > 90` and canonicalizing at `CreateOperation`/`EditOperationParams`:** Dropped because `an_insane_tilt_refuses_at_the_command_door` asserts `A = 120` refuses, `Tilt` is never silently normalized across UI/MCP drafts, and silent angle rewriting can surprise users on 5-axis machines.
  - **Rounding `angles_to_axis` outputs:** Dropped because changing `pose.rs` could affect round-trip tests and CL pose serialization; the bug is strictly the seed tie-breaker in `TiltedFrame::new`.
  - **Only showing plane X/Y vectors in the UI without fixing `TiltedFrame::new`:** Dropped because keeping plane X along the A rotary axis at `A 0 B 180` eliminates the surprise directly and keeps all four bar flats uniform.

- **Cost to change later:** Low. `Tilt::refusal` and `TiltedFrame::new`'s `1e-12` tie-break are small, local helpers; any future UX change to accept non-canonical A on A-only machines only touches `Tilt` and the command doors. Note that stored A0 B180 operations authored under the pre-fix basis (`u = +Y`) would rotate 90 degrees in plane unless regenerated; this matches the safety-floor note on #2133.
