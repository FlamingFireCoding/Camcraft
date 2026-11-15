## #1714 - Plan one release heavy step for each test binary

- Chosen: the release heavy lanes of motion and app are one `--release --no-run` build, then one step for the lib, the bins, the doc tests and each integration binary that can hold an ignored test.
- Why: a binary with no `ignore` in its files runs no test in a heavy step. Planning all of them gave 526 steps, one cargo start each. The plan test pins that every binary with an `#[ignore]` has a step.
- A binary that pulls in other files (`mod x;`, `include!`, `#[path]`) always gets a step. Unreadable source also counts as heavy.
- Dropped: kernel heavy steps. Release never ran kernel `--ignored` before. Adding them is a coverage change, not a split. Left for a follow-up, #1783.
- Dropped: a per-test time limit. libtest only reports "running for over 60 seconds" on stable. The step note lists those tests by name. A real limit needs nightly `--report-time`.
