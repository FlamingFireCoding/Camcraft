## #2127 - Isolate shape-mask diagnostic directories

**Decided:** Resolve Cargo's test temporary directory at compile time with
`env!("CARGO_TARGET_TMPDIR")`. Allocate a diagnostic directory for each call
using the process ID, an atomic counter, and exclusive directory creation.
Retry an existing candidate. Reuse this allocator for explicit unit-test
scratch paths and the shape-mask verifier's default output.

**Why:** Cargo supplies this value at compile time. The runtime lookup was
absent on uma and fell back to a shared OS temporary path. Another user's
existing directory then prevented the overcut test from writing its PNG.
Separate allocations also keep concurrent tests from deleting or replacing
another run's evidence. Exclusive creation handles reused process IDs and
existing candidates. Other filesystem errors still reach the PNG writer's
named diagnostic.

**Dropped:** A fixed OS temporary path keeps the ownership bug. A process ID
alone permits collisions after process-ID reuse and shares output within a
process. A new temporary-directory dependency adds a dependency for a small
test-only helper. Deleting a preexisting directory risks another run's output.

**Cost to change later:** Only the shared PNG test helper and its regression
tests need to change. Toolpaths, mask verdicts, and G-code are unaffected.
