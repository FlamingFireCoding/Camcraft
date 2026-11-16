## #1768 - Pure-Rust 2D shape-mask verifier and RGB diff PNG reporter

**Chosen:** Added test-only shape-mask helpers under
`crates/motion/tests/common/` (`png.rs`, `mask_grid.rs`, `shape_mask.rs`,
`mod.rs`), unit/regression tests in `crates/motion/tests/shape_masks.rs`,
and initial generator mask tests in `trochoid_slot.rs` and
`adaptive_hsm.rs`, plus documented the harness in `AGENTS.md`. The encoder
writes valid uncompressed 8-bit RGB PNG diffs (`IHDR`, zlib stored `IDAT`,
`IEND` with CRC32/Adler32) to `CARGO_TARGET_TMPDIR` (or temp fallback) only
on assertion failure. Engagement uses instantaneous XY arc tangent for arcs
(with chord tangent for linear segments), a `r + 0.75 * cell` rim sample ring
where out-of-expected cells read as `Rim::Open`, flushes completed `>= 1`
turn descending helix arcs cleanly down to floor Z before planar cleanup,
and checks 1-cell border dilation/erosion against `MoveKind::Feed` swept
capsules (`assert_no_overcut`, `assert_bounded_leftover`,
`assert_max_engagement`, `assert_symmetric`,
`assert_within_stock_extension`). The `2D Adaptive` fixture uses a
rounded-corner pocket (`30x16` core offset by `8 mm`) with `adaptive_hsm`'s
`1.35x` debug engagement bound so it runs well under 60 s in debug.

**Why:** Comparing the 2D `Region` mask against the swept cutter footprint
catches spatial escapes, overcuts, disallowed uncut pockets, missed open-end
extensions, symmetry breaks, and stepover spikes directly with a visual RGB
diff PNG, without adding any `Cargo.toml` dependencies (`png`/`image`), touching
product code, requiring a GPU, or committing binary snapshot goldens.
Lagging secant chord directions on tight arcs previously over-counted
contact arcs by ~10 deg and tripped false engagement failures on valid
trochoidal and spiral passes; computing instantaneous arc tangents aligns the
rim probe with actual cutter heading. Flushing completed entry helices
prevents flat bore-floor cleanup arcs from registering phantom full-width
engagement inside already-cleared entry bores.

**Dropped:** Adding an external `png`/`image` dev-dependency (forbidden
without approval), reading full 360 deg rims or `r + 1.0 * cell` (which
counted already-swept behind-center discs or diagonal wall curvature as extra
contact degrees), treating non-cut `Rim::Open` outside `expected` as solid
stock (which fails boundary finish passes), and keeping a `>= 50 s` raw
square adaptive pocket in debug (too close to the 60 s debug test ceiling
and mixes in corner-fallback behavior).

**Cost to change later:** Localized to `crates/motion/tests/common/` and
its three integration test files; `quality::measure` and all product crates
remain untouched.
