## #2516 - Tab, Shift+Tab and Enter move between the value fields of the Properties pane

- **Found:** egui's own Tab already moved the focus, but in draw order over each focusable widget: from a value field it went to a group header (no visible focus mark), a combo or the action bar's buttons. Thus the focus seemed to leave the form. Enter committed the value and dropped the focus (egui's default).
- **Decided:** a value-field ring for the Properties pane (`crates/ui/src/egui_app/layout/field_ring.rs`).
  - The form seam's number helpers in `fields.rs` register each value field, with the group it sits in (`group_open` marks the group).
  - From a registered field, Tab moves to the next field and Shift+Tab to the previous one. The ring wraps at the two ends.
  - Enter commits the value to the draft (egui's DragValue, as before) and moves to the next field of the same group. After the group's last field the focus drops, as before.
  - From a combo, a checkbox or a button, egui's own order stays.
  - **Why:** the issue asks for the next value field, and the form seam is the one place that knows which widgets are value fields.
  - **Dropped:** making headers, combos and buttons not focusable. That breaks keyboard access to them.
  - **Dropped:** changing each page. The helpers cover the pages that use the form seam; a page that adds a raw `DragValue` is outside the ring (see #1122).
- **Decided:** the ring records the target at the end of the pane, asks for a repaint, and focuses the field when the next frame of the pane begins, before any widget draws.
  - **Why:** the field then GAINS the focus, so egui's DragValue selects its text, and a typed number replaces the old one, as after a click. egui also applies its own Shift+Tab at the start of the next frame (`id_next_frame`); the ring's request comes after it.
  - **Dropped:** a request at the end of the frame (the first try). On the next frame the field was not "gained", so its text was not selected and typing appended to the old number. Shift+Tab was also lost to `id_next_frame`.
- **Changed tests:** the `thread_hand_tests` helpers `edit_number` and `type_dip` click a field only when it is not focused. After Enter the next field already has the focus with its text selected; a click on a focused field only places the cursor.
- **Not changed:** the value reaches the document only through the page's command (Apply, `EditOperationParams`). Focus is ephemeral UI state and needs no command.
