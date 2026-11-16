## #2262 - Debug info: frame-rate log rule and a dockable panel

**Decided.**

- The Debug info panel is a dock tab (`Tab::DebugInfo`), not a floating
  `egui::Window`. It opens beside the properties pane like the Machines and
  AI Chat panes. The user can drag it out to a float and back.
- The status strip door sends `LayoutRequest::ToggleDebugInfo`. `layout::settle`
  mirrors the dock into `State::debug_info_open`, so the tab's own X closes
  it too. No command, no history entry (layout is view state).
- The frame rate is read from the frame stamps that the shell already takes.
  A tick is 0.5 s. A gap of more than 250 ms between frames means idle: the
  reading restarts and nothing logs.
- Log rule: a new level (change of 15 percent and at least 5 fps) logs one
  line. If the rate keeps moving, one summary line (min, max, mean) is logged
  at most each 10 s. One step then steady logs one plain line, not a summary.
- The `gpu: slow frame` line uses the same 10 s limit (was 5 s). The two
  lines have one rule, so a stuck loop cannot flood the log.
- The log never requests a repaint. The panel shows "idle" when the last
  reading is older than 1 s.

**Why.** The issue asks for it in these words. The measure is O(1) per frame.

**Dropped.**

- Keeping the floating window and only adding a dock-me button: the
  maintainer asked for docking "as the other panels dock".
- A saved layout: the dock is not saved today (layout is view state, see
  `layout/mod.rs`). The panel is part of the dock, so it will be saved
  when the dock is. This PR does not add dock persistence.
- Frame rate from per-frame fps: too noisy. The tick mean is used.

**Cost to change.** The thresholds are constants in `egui_app/frame_rate.rs`.
