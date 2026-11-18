## #2453 - Run the full shoulder containment regression in the heavy lane

**Decided:** Add a named release heavy-lane ignore reason to
`containment_outside_clears_past_the_edge_never_the_recess`. Keep its full STEP
fixture and every assertion, including the 60-second planner assertion.

**Why:** The fixture exceeded 60 seconds in debug and disrupted quick gates.
The existing gate runs ignored tests in release at the cut, preserving this
real-model regression without running it in ordinary debug checks.

**Dropped:** A smaller fixture would weaken the original real-model scenario.
A wider timing limit would leave the slow work in debug gates. Removing the
timing assertion is unnecessary when the test runs in optimized release mode.

**Cost to change later:** Remove the ignore attribute once the fixture or
planner is proven fast enough for the debug lane.
