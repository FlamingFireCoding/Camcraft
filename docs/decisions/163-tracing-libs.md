## #163 - the last library and UI prints become tracing events

**Decided.** The remaining diagnostics in the library crates (`cl`,
`geometry`, `motion`, `sim`, `tools`) are `tracing` events: refusals and
fallbacks at `warn` (a refused axis or C angle, a multi-axis move that holds
vertical, a corrupt sim path left out, the dual iso-surface falling back to
marching tetrahedra, a missing starter catalog), the opt-in projection
profile (`CAMCRAFT_PROFILE_PROJECTION`) at `info`. Each crate takes
`tracing` and `tracing-subscriber` from the workspace, and each standalone
bin (`camcraft-cl`, `-geometry`, `-motion`, `-sim`, `-tools`) installs a
stderr subscriber with the app's filter rule (`CAMCRAFT_LOG`, else
`RUST_LOG`, else `info`). In the UI, the `diag` seam (#31) emits `tracing`
events natively (target `camcraft_ui`) instead of printing, and the native
window, GPU bring-up, quit hook, frame watch and headless proof now speak
through that seam; only test-only code (`gpu.rs`'s test module,
`headless_gpu.rs`'s `device_for_test`) still prints directly, so a test run
shows its "SKIPPED - NOT a pass" line without a subscriber.

**Why.** One subscriber, one filter and one line format on the launch
terminal; a library line that bypassed it could not be quietened or
raised. Neither crate is new to the workspace (both are workspace
dependencies since PR #222, already in `Cargo.lock`); no `camcraft-*` edge
is added.

**Dropped.** Leaving the standalone bins without a subscriber: the
converted library warnings would then vanish there, which is silence.
A shared `init` helper in a low crate: `tools` does not depend on
`geometry`, and a new `camcraft-*` edge for six lines is not worth it - each
bin carries its own six-line `log_init`. Converting the bins' own
`println!` reports and usage errors: they are the CLI's answer, not
diagnostics. Test-only prints stay prints.

**Cost to change.** Small: the events keep their text; a different filter
default is one string per bin.
