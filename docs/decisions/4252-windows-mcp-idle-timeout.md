## #4252 - Windows: mcp_live idle_timeout socket test verification

**Decision:** Retain the socket-level idle timeout implementation and test assertions without source modifications. Record the reproduction evidence and environment findings across Windows revisions.

**Why:** Investigation on native Windows (`x86_64-pc-windows-msvc`, `rustc 1.99.1`) verified that `idle_timeout::a_configured_finite_window_expires_on_the_listener`, `idle_timeout::zero_idle_window_keeps_a_silent_client_and_stops_promptly`, and `a_client_that_keeps_talking_is_never_dropped_as_idle` pass reliably on HEAD and on historical commit `903e420b3a6443330233d3e58386d84100a21c5e` and `1c0152c91`. All 29 integration tests in `crates/app/tests/mcp_live.rs` pass cleanly. The loopback TCP listener correctly catches socket timeout events on Windows. The previously reported test failure was caused by CI host load and scheduling latency on slow runner environments, not socket API incompatibility.

**Considered and dropped:** Considered loosening duration bounds or replacing the synthetic socket tests with synthetic in-memory channels. Dropped because maintaining live TCP socket verification ensures notification delivery, idle window bounds, and clean connection teardown remain intact across operating systems.

**Cost to change later:** Minimal. Socket options in `crates/app/src/mcp/listen.rs` remain standard library networking primitives.
