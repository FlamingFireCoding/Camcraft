# #2104 - remove dead MIN_LABEL_PX

Decision (unattended, effort:low): delete `MIN_LABEL_PX` from `crates/ui/src/egui_app/painted.rs`
and point the `fit_label` comment at the theme token `pal.font_label_min`.

Considered: keep the constant as the clamp floor. Dropped: the floor already comes from
`pal.font_label_min` (clamped by `clamp::FONT`), so a second hard-coded 10.0 would be a
duplicate source of truth that the theme cannot override. No `#[allow(dead_code)]`.

Pinned by: `cargo clippy -p camcraft-ui --all-targets -- -D warnings` (clean), and the
existing ribbon caption/theme suites in the quick gate.
