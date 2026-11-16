## #2152 - a cache judged under the old rapid law reads Stale (op stamp salt v2 -> v3)

**What was decided**

1. The op stamp's salt moves from `camcraft-op-stamp-v2` to
   `camcraft-op-stamp-v3` (`crates/app/src/operation/stamp.rs`). Every
   cached toolpath made before the change reads Stale once. The post
   refuses a Stale operation by name: "<op>: inputs moved since
   generation - regenerate before posting". A regenerate judges the
   path under the #2121 rapid law, which holds the post if the traverse
   at the clearance plane goes through stock.
2. The salt comment now states the rule: bump the salt when a verify
   law that a cache records changes.
3. `examples/first-router-part.camproj` carries the three v3 stamps.
   Each of its operations was regenerated under the new law: the same
   move count and no warnings, so the new stamps are true. The other
   recorded projects (`examples/first-part.camproj`, the microARC kit,
   the #371 UI fixture) are not touched: no test posts them. They read Stale and
   regenerate one time, like a user's project. (Their files are older
   versions, and a regenerate of `first-part` gives a different move
   count, so a new stamp on the old cache is not true for each of them.)
4. The pin is a recorded project, `crates/app/tests/stale_rapid_law/rapid_law_v2_cache.camproj`:
   the #2121 repro saved at e7050be17 (salt v2) with the new law's
   verdicts taken out of the cache. Before the salt move it reads Ok
   and posts with no warning; after it, the post is refused.

**Why**

- The post gate trusts the WARN records of the cache, and it does not
  re-run the rapid law. A cache from the old law must not read fresh.
- The salt is the smaller of the two known mechanisms. It is one line,
  needs no project-version change, and older builds still open the
  files. The salt has moved before for the same reason (the v1 -> v2
  height change, the M4.3 rename).

**Considered and dropped**

- A project-version migration (v62 -> v63, like v29 -> v30 in
  `project/apply.rs`) that drops stored stamps. It has the same effect
  for the user, but it changes the file version, and older builds then
  cannot open files that this build saves.
- A re-run of the rapid law at post time over each cached path. It needs
  the prior-stock simulation of the program ahead at each post. This is
  a larger change, and it is not suitable while it holds the v0.22.0 cut.

**Cost to change later**

- Every saved operation regenerates one time after the upgrade. This is
  the same one-time churn as each earlier salt or version step.
- A future verify-law change must bump the salt again. A
  `verify_law_version` term in the digest is an option if that becomes
  frequent.
