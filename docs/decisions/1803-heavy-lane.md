## #1803 - Run the L-shaped adaptive engagement test in the release heavy lane

**What was decided.** Move `l_shaped_pocket_is_cleared_under_engagement_limit` to `#[ignore = "heavy lane: release engagement and rest-area measurement"]`. Keep its fixture, both tool diameters, every assertion, and the `#[test]` count unchanged.

**Why.** The test exceeded 60 seconds in debug on macOS. The testing law assigns such measurements to the heavy lane, and the release tier runs the ignored motion tests. This keeps the batch tier fast while preserving the engagement and rest-area checks.

**What was considered and dropped.** A slimmer L-shaped fixture could reduce debug time, but it could also reduce coverage of the existing lane and cleanup behavior. Keeping the fixture and moving its execution preserves that coverage with a smaller change.

**Cost to change later.** Low: remove the ignore attribute after measuring a fixture or implementation that meets the debug time budget while preserving these checks.
