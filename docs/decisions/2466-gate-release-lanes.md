## #2466 - Release-gate queue and suite isolation

**Decided:** In PR #2461, keep builds first and exclusive, replace barrier
waves with a shared work queue, then run exclusive suites one at a time.
Use the existing lane limit and per-lane job budget for shared steps; give
exclusive steps the whole budget. Order the queue longest first using the
newest prior release run's `timings.tsv`. Unknown steps rank with the
longest known step, ties retain plan order, and an empty history retains
plan order. History orders work; it never selects or drops a planned step.

**Why:** A barrier wave waits for its slowest suite while other lanes sit
idle. A queue lets a freed lane take another suite immediately, and starting
long suites early is intended to reduce the final idle tail. No full release
performance measurement is claimed by this decision.

**Isolation policy:** Builds, clippy and every UI step remain exclusive.
App/kernel steps that do not identify one target remain exclusive too.
For single app/kernel targets, retain exclusivity for `lanes::ISOLATED`:
the library targets and named integration suites with watchdogs, polling
deadlines or elapsed-time assertions. Other single targets may share lanes.
The named list was identified from those assertions; new suites with such
assertions must be added. This preserves protection against load-sensitive
failures without serializing every app/kernel target. Failed builds still
stop execution, failed suites still allow the remaining suites to run, and
the checkout guard runs before each queued step.

**Considered and dropped:** Keeping barrier waves leaves capacity unused
when suites have uneven durations. Keeping every app/kernel step exclusive
retains the long serial portion of the release gate. Running every suite
concurrently exposes watchdog and UI tests to interference. Removing the
job budget risks oversubscribing the machine. Requiring timing history
would make a first run depend on a previous run, so absent or unreadable
history falls back to plan order rather than refusing the gate.

**Cost of changing later:** Restoring barrier waves requires changing the
release executor and its overlap/failure/checkout tests. Restoring blanket
app/kernel isolation only requires changing `lanes::alone` and its policy
pins, at the cost of more serial execution. Adding a newly discovered
timing-sensitive suite requires an `ISOLATED` entry and checking its target
exists. A different queue ordering requires updating the ordering tests;
the plan steps and their names remain independent of timing history.
