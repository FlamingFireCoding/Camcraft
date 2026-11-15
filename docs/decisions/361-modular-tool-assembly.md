## #361 - Modular multi-piece tool body assembly and CSYS mating

- **What was decided:** Extend `ToolBody` with `pieces: Vec<BodyPiece>` (keeping `facets` and `frame` as the merged body), serialize `pieces` additively in `ToolBodyDto` (omitted when a body has at most one piece with no CSYS metadata), support rigid frame mating between ISO 13399 STEP coordinate systems (`MCS` <-> `CSW*`, or explicit parent/child CSYS names) alongside `spindle` / `lathe` piece placement via `attach_piece` / `remove_piece`, expose `attach_tool_body_piece` and `remove_tool_body_piece` MCP tools plus a Modular Assembly section in the Tool Editor, and refuse ambiguous or missing CSYS names by name rather than guessing.
- **Why:** Keeping `facets` as the merged assembly preserves every existing collision check, spun-profile calculation, renderer, and fixture with zero runtime traversal overhead, while storing per-piece facet counts, labels, and transformed STEP CSYS entries makes piece attachment, removal, inspection, and undo deterministic without altering single-piece `.camproj` or `.camtools` serialization.
- **What was considered and dropped:**
  - *Evaluating separate runtime facet arrays per piece in simulation and rendering*: dropped because merging facets on rebuild avoids touching collision/rendering hot paths and guarantees the saved STL-backed body matches what simulation judges.
  - *Silently picking the first `MCS`/`CSW` frame when multiple mating ports exist*: dropped because guessing masks mis-oriented assemblies; explicit refusal names the candidate frames instead.
  - *Automatic STEP mesh enlargement (`#1041`) or single-holder `CIP` lathe auto-align (`#1039`)*: kept out of scope for `#361` so multi-piece assembly ships without altering unrelated import tolerances or single-holder workflows.
- **Cost to change later:** Low. `BodyPiece` metadata sits behind an optional `pieces` field on `ToolBodyDto` with facet-count validation on load, so changing how pieces are stored or split later requires only updating `tool_body_assembly.rs` and `tool_body_wire.rs`.

### PR #1744 review fixes

- **What was decided:** Require the saved pieces' concatenated facets to equal the validated combined mesh exactly, in order. Add an editor target-piece selector that passes the chosen piece index to mating while keeping `None` as “Auto (all pieces)”.
- **Why:** Attachment and removal rebuild from the pieces, so count-only validation could accept coordinates that later shrink the collision envelope. Exact equality matches the existing concatenation used when saving and rebuilding. Selecting a piece disambiguates repeated CSYS names in three-piece assemblies without changing automatic mating for unambiguous assemblies.
- **What was considered and dropped:** Tolerant coordinate comparison could permit a smaller collision envelope; canonicalizing or discarding either saved mesh would silently repair inconsistent geometry instead of refusing it. Always selecting the first piece would hide ambiguous mounts and remove the existing automatic search.
- **Cost to change later:** Low. Validation stays in the shared project/tool-library DTO loader; target selection is ephemeral editor state passed to the existing undoable attachment command.

### PR #1744 reapply review fix

- **What was decided:** Retain all piece records in the editor draft. Apply the same placement to every piece mesh and its coordinate systems, then validate the combined geometry and tool fit through `SetToolBody` before recording the command.
- **Why:** Applying an unchanged assembly must preserve each piece's identity and mating frames. Changing placement must move those frames with the geometry so adapters remain removable and selectable as mating targets.
- **What was considered and dropped:** Flattening the assembly into one piece loses its separate records and adapter coordinate systems. Cloning the landed assembly only for unchanged Apply would leave placement changes destructive and would not move all piece geometry and frames together.
- **Cost to change later:** Low. Piece retention belongs to the ephemeral draft, and placement and validation stay behind the existing undoable body command.
