## #1627 - scope the program sim's mark credit to the part(s) the operation marks

**Decided:** `stock_sim::Mark` carries `target: Option<&'a [Triangle]>`, and `operation::mark_law::sim_mark_of` pairs `budget_of` with `target_facets(world, op)`:
- For a stock-source Engrave or Trace (`SrcRef::Stock`, `SrcRef::Entity(stock_name)`, or a `Selection` containing the stock or empty), `target` is `None`, so `credit_marks` credits every setup part (`facets`), matching `generate::run_job` (`parts_marked`).
- For a part-source Engrave or Trace (`SrcRef::Entity(part)` or a `Selection` on part entities), `target` is `Some(facets)` holding only those judged source entities' facets in the setup's frame (`setup_align::facets_in`, filtered by `stock_source::judged`).
- In `credit_marks`, a mark with `target: Some(parts)` skips empty targets, computes `part_surface` and `undersides` over `parts` itself, and credits column `k` only when `parts` owns the top surface at `k` (`(p - target_top[k]).abs() <= 1e-9`), bounded by `parts`' own underside (`MARK_FLOOR`).

**Why:** Previously `credit_marks` credited every declared mark against the union of all setup parts (`model_facets`), so a part-source mark on B whose flank entered a neighbouring part A by up to the mark depth was forgiven on A by the program sim while the op's own neighbour law (#1621) convicted it. Scoping each mark to its target parts makes the sim agree with the operation, and using `parts`' own undersides ensures a mark on B stacked over A is bounded by B's own floor rather than A's.

**Considered and dropped:**
- Filtering `credit_marks` by entity IDs instead of `&[Triangle]`: `camcraft-sim` has no ECS or entity-name model; passing `Option<&'a [Triangle]>` keeps `Mark` `Copy` and pure geometry within `camcraft-sim`.
- Changing `mark_law::budget_of`'s return type: keeping `budget_of -> Option<f64>` and adding `target_facets` + `sim_mark_of` preserves the existing public signature while exposing the target mesh alongside the budget.

**Cost to change later:** one field on `stock_sim::Mark`, `mark_law::target_facets`, and the one wiring site in `stock_sim_marks::program_marks`.
