## #1799 - Pin the modular STEP fixtures

**What was decided:** Add measured verdict rows and per-file tests for `modular_clamp_unit.step` and `modular_lathe_head.step` to the existing STEP agreement bench. Both importers agree at one millimetre per unit, and both meshes are watertight within the existing tolerance.

**Why:** The fixture-set assertion already requires every STEP file to have a verdict row and a test. The modular fixtures were added without those entries. Measuring them preserves the bench contract without changing production geometry or tolerances.

**Considered and dropped:** Extending the optional quick-gate fixture mapping would add process logic to this mechanical test-data fix. Keep that as a separate mapping change. Do not omit the fixtures or relax the fixture-set assertion.

**Cost to change later:** Update the two verdict rows if a separately justified importer change alters their measured results. Adding fixture-to-suite gate mapping later is a separate tooling task.
