## #2512 - Operation notes documentation and module boundaries

**Chosen:** Document the optional per-operation `notes` array returned by
MCP `generate` and `verify`, including its string values, omission when
empty and informational meaning. Extend `unreached.rs`'s module comment
to describe both the narrow-band check and the corner check, including
the stock-outline intersection and approximation-noise filter.

**Why:** These public descriptions lag the behavior added by #2503.
Explaining existing output needs no change to toolpaths or posting.
Leaving the descriptions unchanged was rejected because an agent could
miss uncut material even when generation and verification succeed.
Later wording changes cost a documentation edit and its contract checks.

**Chosen:** Retain `crates/app/src/operation/generate.rs` and
`crates/app/src/script/api.rs` in their current modules for this issue.
Both remain over the approximately 400-line limit; this decision records
the remaining structural debt rather than claiming that either was split.

**Why:** #2512 explicitly accepts a recorded decision for each owed item.
Splitting the generation dispatcher and its verification tail requires
careful handling of shared geometry, frame and verification state.
The script API has a clearer prospective seam between document/tool IO
and generation, verification and reporting, but extracting that seam is
separate behavior-preserving work. Broad extraction was considered and
dropped from this small documentation correction because it would widen
the code and validation scope without changing the documented output.
Changing this decision later costs scoped module extraction, import and
visibility updates, and the existing generation/script regression tests.

**Chosen:** Keep the stock-outline selection in
`generate.rs::run_job_arms`; continue passing its borrowed result to
`keep_law::narrow_noted`.

**Why:** The dispatcher owns the cached stock region, the optional local
frame and the temporary projected outline. It can reuse the cached
outline when no frame transform is needed and retain the temporary
outline for the duration of the note check otherwise. `keep_law` owns
the decision to attach band and corner notes, not generation-frame
preparation. Moving selection into `keep_law.rs` was considered and
dropped because it would expand that helper's context or require owned
geometry where a borrow currently suffices. Changing this later costs
an explicit input/lifetime interface and regression checks for stock
and model sources with and without a local frame.
