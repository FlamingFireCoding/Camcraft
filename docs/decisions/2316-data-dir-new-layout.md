## #2316 - The data folder lookup knows the new cargo build-dir layout

**Decided:** `camcraft_post::data_dir::cargo_profile_dir` accepts two
layouts. The old one: the exe is in `target/<profile>` or its
`deps` / `examples` child, and the profile folder holds `.fingerprint`. The
new build-dir layout: a test exe is in
`target/<profile>/build/<pkg>/<hash>/out`, and the profile folder holds
`build/camcraft-post` (the unit folder of this crate, which every exe that
reads data files links). The uplifted binaries in `target/<profile>` are
known by the same `build/camcraft-post` marker.

**Why:** the post-merge runner's cargo uses the new layout. There the test
exes are in `.../out/` and `target/debug/.fingerprint` does not exist, so
the lookup tried only `<exe dir>/assets/<kind>` and about 200 test binaries
failed (run 43990986558). The local cargo (1.99.0) still uses the old
layout, so Windows worktrees passed.

**Dropped:**
- An env var (`CARGO_MANIFEST_DIR` at run time): the gate runs some test
  exes outside cargo, and an installed exe must never read it.
- A compiled-in copy of the assets for tests: #2293 removed it by intent.
- Any marker that an installed folder can hold by chance (a bare `build`
  folder): the review of #2293 requires that an installed exe never reads
  the build machine's checkout.

**Cost to change:** one function and its tests in `crates/post/src/data_dir.rs`.
If cargo changes its layout again, add the new shape there.
