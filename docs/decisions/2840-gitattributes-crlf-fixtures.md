## #2840 - pin golden, machine, STEP and workflow fixtures to LF in .gitattributes

- **Chosen:** pin all test fixtures, golden post outputs, machine definitions, STEP/STP models, agent and skill definitions, and GitHub workflows to LF line endings (`text eol=lf`) in `.gitattributes`.
  - Why: Git for Windows checks out text files with CRLF by default (`core.autocrlf=true`). Test suites comparing post programs byte-for-byte (such as `explain_fanuc_family`), parsing machine definitions (such as `wire-iso-fixture.mch`), importing STEP models (`import_step_curve_guard`), and xtask tests parsing skill markdown or workflow concurrency blocks expect and require LF line endings (`\n`).
  - Dropped: modifying individual tests to accept CRLF. Normalizing line endings at the repository boundary via `.gitattributes` guarantees deterministic behavior across platforms without weakening byte-exact golden test assertions.
- **Pinning tests:**
  - `xtask/tests/example_line_endings.rs`: added `all_golden_and_fixture_files_have_eol_lf` asserting that `git check-attr eol` reports `lf` across all golden, recorded, machine, STEP, skill, and workflow directories.
  - `xtask/tests/example_line_endings.rs`: added `golden_and_fixture_files_keep_lf_with_autocrlf` asserting that checked-out files maintain LF line endings without carriage returns under `core.autocrlf=true`.
  - Verified affected suites pass cleanly: `explain_fanuc_family`, `wire_taper_angle`, `import_step_curve_guard`, and `post_merge_concurrency`.
