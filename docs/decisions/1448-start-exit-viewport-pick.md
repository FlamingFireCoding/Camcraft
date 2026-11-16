## #1448 - 2D Contour start and exit viewport pick

- **What was decided**:
  1. Keep `start_exit_pick: Option<StartExitRole>` on `OpUi` (`crates/ui/src/state.rs`) and pass arm state/actions between `op_page.rs` and `start_exit_rows.rs` through `egui::Context` temp data (`armed_id` / `act_id`), rather than adding another top-level field on `State` or threading `Option<StartExitRole>` through `params_page.rs`.
  2. Accept faces, edges, and corners (`PickKinds { faces: true, edges: true, points: true, centres: false }`) regardless of toolbar `GeoPick` mode, write `found.at` XY directly into `OpUi::params`'s `links.approach.start_point` / `exit_point` as a draft edit (committed and undoable via `Apply` / `Generate` / `EditOperationParams`), keep picking armed across clicks until `"Stop picking"`, `"Clear ..."`, tab change, Apply/Generate, or closing the editor, and refuse fixture clicks and tilted contours by name in `state.status`.
  3. Draw distinct screen-space markers in `pick_ink::paint` while the Contour Lead tab is open: a diamond (`Overlay::Pick`) for `start_point` and an X-cross (`Overlay::Region`) for `exit_point` projected at the contour's resolved `top_z`.
- **Why**:
  - Placing the flag on `OpUi` automatically clears the armed pick whenever `OpUi::from_op` reloads after Apply/Generate/Undo or target switch, and avoids touching `State::new`, `params_page.rs`, or `shell.rs` alongside open PRs (`#1330`, `#2185`, `#2093`).
  - Writing to the page draft matches `start_exit_rows` (typed X/Y) and lets users inspect the marker in the viewport and adjust clicks without cluttering undo history, while `sane.rs` continues to own the Contour-only and untilted domain law at commit time.
- **Considered and dropped**:
  - *Restricted vertex-only picking*: walls of a 3D contour often have no vertex at the desired mid-edge lead-in/lead-out spot; taking the surface/edge/vertex hit point `found.at` lets users click anywhere along a wall.
  - *Immediate `EditOperationParams` on viewport click*: dropped because multi-click adjustment would pollute undo history and fight the draft model on the rest of the operation page.
- **Cost to change later**: Low and local to `crates/ui/src/start_exit_pick.rs` and `crates/ui/src/egui_app/{op_page,start_exit_rows}.rs`; moving `start_exit_pick` to `params_page` arguments or adding Pocket/Adaptive support later touches only those UI bridge files.
