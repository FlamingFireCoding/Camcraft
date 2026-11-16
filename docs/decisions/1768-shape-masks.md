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
where out-of-expected cells read as `Rim::Open`, and records the deepest
actual cutter position covering each cell. Ramps are clipped to the
selected depth interval for shape coverage. Descending helices retain
their actual depths; a completed turn never promotes the entire footprint
to its endpoint floor. The helper checks 1-cell border dilation/erosion
against swept
capsules (`assert_no_overcut`, `assert_bounded_leftover`,
`assert_max_engagement`, `assert_symmetric`,
`assert_within_stock_extension`). The `2D Adaptive` fixture uses a
rounded-corner pocket (`30x16` core offset by `8 mm`) with `adaptive_hsm`'s
`1.35x` debug engagement bound so it runs well under 60 s in debug.
Both generator engagement fixtures explicitly start with a predrilled
8 mm radius starter bore at the final floor: the Adaptive bore is centred
at `(0, 0)`, and the trochoidal bore at `(-20, 0)`. Each radius is the
3 mm entry orbit plus the 5 mm cutter radius. `evaluate_with_precleared`
initializes clearance only inside that region and only to the stated
depth. It gives no swept coverage credit, so overcut, leftover, symmetry
and stock-extension assertions still judge the generated path itself.

**Why:** Comparing the 2D `Region` mask against the swept cutter footprint
catches spatial escapes, overcuts, disallowed uncut pockets, missed open-end
extensions, symmetry breaks, and stepover spikes directly with a visual RGB
diff PNG, without adding any `Cargo.toml` dependencies (`png`/`image`), touching
product code, requiring a GPU, or committing binary snapshot goldens.
Lagging secant chord directions on tight arcs previously over-counted
contact arcs by ~10 deg and tripped false engagement failures on valid
trochoidal and spiral passes; computing instantaneous arc tangents aligns the
rim probe with actual cutter heading. A descending helix does leave stock
at its final floor; its first floor orbit cannot be assumed to satisfy
the subsequent radial engagement bound. An explicit starter bore makes
these fixtures test side cutting under the unchanged Adaptive and
half-diameter trochoidal bounds. The bore's radius and depth are stated
starting-stock conditions, not clearance inferred from a future cut.
Pins reject a smaller or shallower bore, retain overload detection
outside the bore, and keep the solid-stock ramp and helix regressions.

**Dropped:** Adding an external `png`/`image` dev-dependency (forbidden
without approval), reading full 360 deg rims or `r + 1.0 * cell` (which
counted already-swept behind-center discs or diagonal wall curvature as extra
contact degrees), treating non-cut `Rim::Open` outside `expected` as solid
stock (which fails boundary finish passes), and keeping a `>= 50 s` raw
square adaptive pocket in debug (too close to the 60 s debug test ceiling
and mixes in corner-fallback behavior). Also dropped completed-helix floor
promotion (it invents clearance), ignoring either generator test, and
raising engagement limits to accommodate entry-floor cutting (neither
would preserve the radial-load check).

**Cost to change later:** Localized to `crates/motion/tests/common/` and
its three integration test files; `quality::measure` and all product crates
remain untouched.
