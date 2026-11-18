## #2469 - Name the blocked cause of the whole-model heavy pin

**Decided:** Name blocked issue #1950 and its expected red result in the whole-model heavy test's ignore reason. State that stock-ring plans exceed optimal load and fall back to Offset rings. Preserve all three tests and every test body and golden assertion. Keep the temporary metadata pin outside the branch so the test count stays three.

**Why:** The release run on upstream/main 7bcc13a4b failed at the no-fallback assertion. Levels Z 15, 10, 5 and 0 reached loads of 19.2%, 25.9%, 19.5% and 19.5%, above the 14.6% optimal load. The ignore reason must distinguish this known failure from an ordinary heavy lane. The metadata pin and its restore-old-code mutation passed their red/green checks. The quick gate and app all-targets Clippy passed.

**Considered and dropped:** Changing geometry or the planner belongs to held issue #1950 and exceeds this annotation fix. Weakening the assertions would hide the regression. Adding a permanent metadata test would change the required test count.

**Cost to change later:** When #1950 is resolved, rerun the release pin and remove the expected-red explanation once it passes. No planner or golden migration is needed for this annotation.
