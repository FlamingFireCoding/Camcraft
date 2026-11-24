## #3767 - solidcore fork synced to upstream main, pinned at fa67b3f5

**What was decided.** The fork NeoTech/solidcore `main` was 0 commits ahead
and 300 commits behind tangency-labs/solidcore `main`. It took the upstream
commits with GitHub's merge-upstream call, which was a fast-forward (no
force-push, no merge commit needed). `crates/kernel/Cargo.toml` pins all six
solidcore crates at the full rev `fa67b3f560b7a8d59e99cb728f72f85e91dee2da`
(was `2016386442dfee3b2a0c3a1bfdd4017c5254dac8`). `Cargo.lock` changed only
for the six solidcore packages. The workspace line `truck-polymesh = "=0.6.0"`
is removed (#3742): no crate manifest uses it. `truck-polymesh` stays in
`Cargo.lock` as a transitive dependency of `truck-meshalgo`.

**Why.** The maintainer asked for the update (2026-11-23). A fast-forward
keeps the fork's history equal to upstream, and a full rev keeps the build
reproducible.

**Considered and dropped.** A merge commit on the fork: not necessary, the
fork had no commits of its own. A branch reference in place of a rev: the
issue asks for a full rev. A tag on the fork for the pin (as the older
`camcraft-pin-2026-11-11`): not necessary, the rev is on `main`.

**Cost to change later.** One rev string in six lines and `cargo update -p`
for the six solidcore packages.
