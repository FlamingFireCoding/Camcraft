## #2416 - Show heights along the tilted tool axis

**Decided:** Reuse `operation::fold_height` from PR #2418 for the viewport
planes, height echoes, and `operation::region_z`. Stack this change on
PR #2418 because it provides the shared resolver.

Keep `HeightPlane.z` in the resolver's fold coordinate. Subtract
`frame.origin.z` once when `TiltedFrame::to_world` places the plane. For a
non-upright tilt, show the local distance with the words "along tool axis".
Pass the draft tilt to the height rows so edits update the display.

Project the selected setup's stock facets into the plane's local XY frame.
Use the shared `TiltedFrame` and add the tool radius plus 2 mm around the
extent. Keep the untilted display behavior unchanged. Hide planes for an
invalid tilt and use the `checked_frame` refusal text in the height echo.
Place label anchors with the same local-to-world transform as the corners.
Keep the tilted Top rectangle when an old setup-Z outline exists: that
horizontal outline cannot replace the tilted plane.

**Why:** The display must use the same reference heights and frame as
generation. A second resolver could show a different value from the one
that generation uses. Local stock extents keep the plane rectangle over
the stock when the tool axis turns.

**Considered and dropped:** A separate geometry or height resolver would
duplicate the generation rules. Setup-Z rectangles would still show the
wrong plane under a tilt. Generation and G-code changes belong to the
parent PR. Changes to `region_lines` and `shown_regions` would overlap
PR #2256; this change corrects their shared `region_z` read instead.

**Cost to change later:** The UI keeps the display conversion in a shared
helper. A later resolver API change must update that helper and the
`region_z` read together. This decision adds no crate dependencies.
