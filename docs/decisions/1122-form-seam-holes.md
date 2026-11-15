## #1122 - UI form seam: holes family (`holes_page.rs`, `thread_page.rs`)

- **What was decided:**
  1. Extended the form seam with `layout/controls.rs` (`button`, `small_button`, `list_row`, `radio_value`, `radio_pair`), re-exported through `layout/mod.rs` and `fields.rs`, and added `fields::int_range_row` (which wires `labelled_by` from `row_label_id`).
  2. Migrated the `holes` page family (`holes_page.rs` and `thread_page.rs`, covering Drill, Bore, and Thread) to draw exclusively through those form-seam helpers, factoring out the duplicated Drill/Bore axial hole loop into `axial_hole_rows`.
  3. Added `layout/form_seam_tests.rs` to pin zero raw `ui.button`/`small_button`/`checkbox`/`label`/`add`/`add_enabled`/`add_sized`/`radio_value` calls in migrated families (`holes` = 0) and cap the other six unmigrated families at their current baselines so new raw calls fail the guard.

- **Why:**
  - `layout/form.rs` is already 387 lines (near the 400-line file-size limit), so placing the new basic controls in a dedicated `layout/controls.rs` module keeps `form.rs` under 400 lines while keeping `super::layout::*` and `super::fields::*` as the single import seam.
  - Putting `int_range_row` in `fields.rs` alongside `num_row` and `num_plain_row` automatically binds AccessKit `labelled_by` to the enclosing `prop_row` label without visual changes.
  - Migrating one page family (`holes`, zero open PR overlaps) first keeps the PR small, reviewable, and free of merge conflicts with open UI PRs.

- **What was considered and dropped:**
  - Appending the new helpers directly to `layout/form.rs` (dropped to stay below the ~400-line file-size limit).
  - Migrating all 7 page families in one PR (dropped per #1122's explicit "one PR per page family" split).

- **What it would cost to change later:**
  - Trivial: `layout/controls.rs` functions are thin `pub` wrappers under `layout/` and can be backed by any kit widget (or merged/split) without touching page callers.

### PR #1724 review fixes

- **Chosen:** Merge current main and update the tools-and-machine raw-call cap
  from 202 to its merged baseline of 204, and the multi-axis cap from 9 to
  its merged baseline of 16. Keep the holes-family zero-call guard
  and all other family caps. This fixes the reported merge failure without
  weakening unrelated existing guards; dropping those guards or adding arbitrary
  slack was considered and rejected.
- **Chosen:** Retain `docs/reviews/1724/` as review evidence, matching existing
  repository evidence in `docs/reviews/1756/`, `docs/review/1765/` and
  `docs/pr-1711/`. The PR description already embeds all six images. Removing
  these files would discard their capture provenance and link-validation script.

### PR #1724 latest review: scope the guard to migrated pages

- **Chosen:** Keep the hard zero-call limit for the holes family only. Remove
  the unrelated family baselines, and pin the guard's scope to holes_page.rs
  and thread_page.rs. This supersedes the earlier choice to retain all caps.
- **Why:** Main's wire-page additions exceeded the multi-axis cap again.
  Unmigrated page work must not block this holes-only migration.
- **Considered and dropped:** Raising the multi-axis cap to 18 or adding
  arbitrary slack would only postpone the same failure after future changes.
