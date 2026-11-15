## #1703 - Run independent release-gate lanes together

**Decided:** Prebuild the debug test targets and the release heavy targets before
running release-gate suites. Run independent suites in waves of at most four,
mixing debug and release steps. Keep app, kernel, UI, build and clippy steps alone.
Keep invoking Cargo for each suite after the prebuilds, with the existing working
directories, libtest arguments and separate numbered logs.

**Why:** This preserves the existing suite selection and diagnostics while
overlapping test execution. App and kernel contain import watchdogs; keeping their
whole suites alone avoids a fragile list of individual timed tests. UI suites can
open windows and must also run alone. Debug and release use different profile
directories. A temporary probe with four prebuilt two-second test runs measured
8.086 seconds serially and 2.052 seconds concurrently. Cargo briefly held package
and build locks, then released them before test execution. This supports using the
existing Cargo commands rather than writing an artifact parser and test launcher.

The gate reports each lane's duration, total elapsed time and the sum of lane wall
times. The sum is an estimate of serial cost under the current load, not a measured
serial release benchmark. A full release-tier serial/parallel comparison has not
been exercised by this issue's quick verification.

**Also decided:** Compare corresponding removed and added Cargo.lock lines after
normalizing CRLF line endings. Permit unchanged lines and version-line changes;
continue refusing changed names, checksums and other content.

**Considered and dropped:** Running every suite concurrently risks watchdog and
window failures. An unbounded worker pool risks exhausting machine resources.
Splitting individual app/kernel binaries can improve utilization later but needs
a reliable inventory of timed suites. A custom Cargo artifact parser duplicates
Cargo's target selection. Splitting work between uma and m3 adds coordination,
artifact transfer and failure handling beyond the optional multi-host request.

**Cost to change later:** The wave planner and its tests own concurrency and
isolation policy. Finer isolation or a queue that immediately fills freed slots
can change that module without changing suite commands. Multi-host execution
would additionally need a runner protocol and combined checkout/log validation.
