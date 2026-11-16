## #1991 - Named layout tests for windows, modals, and the Machines pane

**Decided (unattended, Antigravity (len)):**

1. **Stack on PR `#1996` in `crates/ui/src/egui_app/tests/page_layout.rs`.**
   PR `#1996` creates `page_layout.rs` (not yet on `upstream/main`). Extending that
   file directly keeps all `layout_*` tests in one module. Dropped: adding a
   second layout test file on `upstream/main`, which would split the refit
   contract introduced in `#1953`.
2. **Use `Role::Window` rects for `egui::Window` and area rects for `egui::Modal`.**
   `Post refused` and `G-code - program` are `egui::Window` frames, so
   `assert_window_layout` checks that the window rect stays inside the screen
   and that anchors stack top-to-bottom inside that window. The unsaved prompt
   and the fixture dialogs use `egui::Modal`, whose `Area` exposes no
   `Role::Window` or `Role::Dialog` AccessKit node. `assert_modal_layout` reads
   the modal rectangle with `ctx.memory(|m| m.area_rect(egui::Id::new(id)))`.
   It checks the frame against `ctx.content_rect()`, checks that anchors stay
   inside the modal and stack top-to-bottom, and checks the dismiss button
   inside the modal separately. Dropped: checking anchors against the screen
   alone, which misses anchors escaping their modal; modifying UI modal widgets
   solely to inject test container nodes.
3. **Verify same-row modal buttons outside the strict vertical stack.**
   In `Unsaved changes`, `Don't Save` and `Cancel` share a button row, so
   `layout_window_unsaved_prompt` checks `Don't Save` in the vertical stack and
   checks `Cancel` containment separately.
4. **Pick labels unique across the seeded shell behind modals.**
   The seeded `Operation: Contour1` page behind an open modal already contains a
   `Close` widget, which makes `query_by_label("Close")` ambiguous.
   `layout_window_fixture_library` queries `"Delete from library…"` instead, and
   `layout_window_unsaved_prompt` queries `"Don't Save"` rather than `"Save"`.
5. **Isolate `layout_window_machines` from user machine files.**
   Set `machines_pane.user_dir = Some(None)` before toggling `Panel::Machines`,
   matching the existing `machines_pane` unit tests, and assert anchors inside
   `layout::leaf_rect(&dock, &Tab::Machines)`.

**Cost to change later:** low. Each test is a short anchor list in
`page_layout.rs`. If UI modals gain dedicated AccessKit dialog nodes later,
update `assert_modal_layout` to query that container rect.

**Review refresh (2026-11-16, Codex):** Keep upstream's `stock_pane_delete`
helper when resolving `stock_fit.rs`. Both sides select Delete inside
Properties; upstream also asserts that this button is unique. Retain all four
stock-fit tests, including the fit-refusal and feedback checks. Dropped: the
branch's equivalent inline lookup, which would duplicate the helper and omit
its uniqueness assertion. No product logic required conflict resolution.
