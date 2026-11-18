## #2422 - Keep the shipped first-part programs current

**Chosen:** Regenerate `examples/first-part.nc` from its shipped project and add
`examples/first-router-part.nc` from its shipped project. Pin both programs with
tests that run the public CLI's `process` lane, including load, generation,
posting and verification, before comparing the complete program text.

**Why:** The mill example kept its original program while posting evolved. The
router example shipped no program. Users need programs that reproduce with the
current application. Running generation also prevents old cached paths in a
project from hiding a change.

**Dropped:** Changing the post implementation would change working behavior to
match a stale example. Copying the catalog's program would not prove that the
guide's project generates it. Comparing selected lines would miss drift elsewhere
in either program.

**Cost to change later:** Regenerate and review the two programs after an
intentional output change. The pinning tests document the CLI command and check
the entire generated result. No product code or dependencies change.
