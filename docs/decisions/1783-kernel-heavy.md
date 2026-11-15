## #1783 - Kernel binaries join the release heavy plan

**Decided:** Add kernel to the existing heavy-crate list. Use the same
release prebuild and per-target steps as motion and app. A unit test scans
kernel test sources independently of that list and checks that every binary
with an ignored test has exactly one release step with the right selector.

**Why:** Kernel's ignored tests had no release gate step. This adds coverage
for them and lets a failed kernel binary run again on its own. Kernel also
joins the existing lib, bins and doc targets, so later ignored tests there
receive the same coverage as the other heavy crates.

**Dropped:** A separate kernel planner would duplicate the target selection
and prebuild logic. Hard-coding today's two ignored integration binaries
would miss later additions.

**Cost to change later:** Change the heavy-crate list or shared planner and
update the coverage pin if the release policy changes. Actual heavy kernel
tests were not executed for this planner change; validation uses unit tests,
an old-planner mutation, clippy and the release-plan dry run.
