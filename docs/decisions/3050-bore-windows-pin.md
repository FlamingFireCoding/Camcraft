# 3050 - bore heavy-lane pins after the solidcore mesh

- Chose: Windows x86_64 pin becomes (3158, 3130, 1141), equal to the Linux
  x86_64 and macOS arm64 pins already on main (via 2903f4dda / 4be274f79).
- Measured (release, `pocket3d_hsm -- --ignored --exact
  the_users_bore_roughs_adaptive_level_by_level_and_verifies_clean`) at
  upstream main d66400b78: Windows x86_64 (3158, 3130, 1141); macOS arm64
  (3158, 3130, 1141); Linux x86_64 (3158, 3130, 1141) by sn3ro on 3c761b5bb
  (comment on #3050).
- Why: PR #2904 changed this fixture's mesh from 952 to 6228 facets at the same
  chord cap; the planner is unchanged. On Windows the verify-clean,
  every-level-cuts, no-fallback and no-budget assertions all passed before
  the count pin failed, so the plan still verifies clean; only the counts moved.
- Considered and dropped: a band for Windows (the pin stays exact per
  platform); loosening any assertion (none touched).
- Added `the_windows_x86_64_bore_count_pin_is_exact`, which rejects the old
  triple and every off-by-one.
