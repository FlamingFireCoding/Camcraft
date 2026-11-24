## #3339 - Operation-level picks for toolpath start and exit positions

**What was decided.**
Move toolpath start and exit picks from `Links::approach` to operation-level `PathPicks` across `OpParams`:

1. **Operation-level storage:** Store `picks: PathPicks` directly on each operation parameter variant (`OpParams::*`). `PathPicks` contains `start: Vec<PathPick>` and `exit: Vec<PathPick>`.
2. **Project format migration:** Load legacy `.camproj` files with `start_point` and `exit_point` under `links.approach`, or legacy `WireContour` with `start` coordinates, and migrate them to the top-level `picks` field. In `OperationDto`, support both `"src"` and `"source"`.
3. **Digest and caching:** Stamp operation picks in `OpParams::stamp_picks` with distinct markers (`0xC7` for start, `0xC8` for exit), and stamp `WireContour` condition and extra picks while preserving legacy digests when picks are empty.
4. **MCP surface:** Expose `picks` with `start` and `exit` points in MCP operation edit and add commands, validating against pick capabilities per operation type.

**Why.**
User picks for toolpath start and exit points are an operation-level specification rather than a property of level link approaches. Putting picks on `OpParams` allows operations without level ladder linking (such as Wire EDM, Drilling, and 2D contours) to share a uniform pick mechanism.

**Considered and dropped.**
- Keeping picks nested in `Approach`: Dropped because non-linking operations do not use `Approach` and would require artificial link structures.
- Breaking backwards compatibility with earlier files: Dropped; migration logic translates legacy wire and approach picks into `PathPicks`.

**Cost to change later.**
Low; `PathPicks` is generic and extensible to multi-point chains and 3D pick coordinates without further schema changes.
