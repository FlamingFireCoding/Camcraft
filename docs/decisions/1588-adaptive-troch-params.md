## #1588 - 2D Adaptive (Trochoidal): engagement limit, return mode, return feed and loop width

**What was decided.**
- Added `AdaptiveTrochParams` (`max_engagement`, `ret: Option<TrochoidReturn>`, `return_feed: Option<f64>`, `loop_width: Option<f64>`) to `camcraft_motion::machining2d::adaptive_troch`, exposed `adaptive_trochoidal_with`, and stored a single `troch: AdaptiveTrochParams` field on `OpParams::Adaptive2D`.
- Reused `ParamsDto::Adaptive2D.max_engagement` for M11.4+ files (`optimal_load.is_some()`), keeping pre-M11.4 legacy `optimal_load: None` files mapped to Spiral's load conversion while defaulting `troch.max_engagement` to `90.0` deg; added `return` (`AdaptiveTrochReturnDto`: `auto`, `microlift`, `retract`), `lift`, `return_feed`, and `loop_width` with `#[serde(default)]` / `skip_serializing_if = "Option::is_none"` so older files default to today's path.
- Placed `Max engagement`, `Loop width`, and `Return` (+ `Microlift height` when `Microlift`) on the 2D Adaptive `Passes` tab and `Return feed` on the `Feeds` tab, rendered only when `Method = Trochoidal`.
- Stamped `troch` into the staleness digest (`0xA6`) only when `Method = Trochoidal` and `troch != AdaptiveTrochParams::default()`.

**Why.**
- Grouping all four fields in `AdaptiveTrochParams` lets the motion planner, `OpParams::is_sane` / `named_refusal`, and `mill2d_page` (`400`-line limit) carry one parameter value without widening pattern matches or adding helper boilerplate.
- Reusing `max_engagement` on `ParamsDto::Adaptive2D` matches `TrochoidalSlot` and `list_operation_types` / `edit_operation` naming directly without breaking pre-M11.4 Spiral migrations (`optimal_load: None`).
- Gate-skipping default `troch` values in `stamp_with` preserves every existing Spiral and default Trochoidal staleness hash from #1509.

**What was considered and dropped.**
- Adding a separate wire key such as `troch_max_engagement` instead of reusing `max_engagement`: dropped because it diverges from `TrochoidalSlot` and MCP's natural `max_engagement` key, while `optimal_load.is_some()` cleanly disambiguates pre-M11.4 Spiral files from modern `Adaptive2D` files.
- Showing trochoidal return/engagement fields greyed-out under `Method = Spiral`: dropped per the page rule and #1588 acceptance criteria that they appear only when `Method = Trochoidal`.

**Cost to change later.**
- Low: `AdaptiveTrochParams` is self-contained and only read under `AdaptiveMethod::Trochoidal`; `ParamsDto::Adaptive2D` fields are additive with serde defaults.

**PR #1782 review remediation.**
- Merged upstream main and retained both the trochoidal parameter set and the corner-arc parameter set, including main's refusal of arc turns with the Trochoidal method. Dropping either parameter set or bypassing that refusal was rejected because saved projects and generation must keep both laws.
- Captured the actual egui shell for Passes and Feeds with Trochoidal selected. Stored review pictures only on the fork's images-only `pr-screenshots` branch and linked them from the PR body. Keeping screenshots or capture provenance on the PR branch was rejected because review evidence must stay outside the product branch. The before pane uses the original PR parent's rendering body; the after pane uses the merged shipping body. A temporary signature adapter lets the original rendering body read the current grouped method field without changing its widgets.
