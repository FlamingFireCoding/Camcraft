## #2970 - multiaxis: Rotary Parallel leaves circumferential band uncut around cylinder

**What was decided**

The shallow/steep split in `build_sups` (`crates/motion/src/finishing3d/support.rs`) now reads the ORIGINAL facet normal (`normal.z >= SPLIT_MZ`) instead of the sky-flipped normal (`m.z >= SPLIT_MZ`). A bottom-facing facet (original `normal.z < 0`) cannot be machined from above in the laid frame, so it is classified as steep and left to the rings. The flipped `m` is kept only for the tangent computation of shallow facets, which all face up (original `m.z >= SPLIT_MZ > 0`, so no flip occurs for them).

The earlier `scan.rs` steep-edge changes (commit `8e5b449b5`) are reverted — they caused wrong retract positions on laid bars. `scan.rs` is back to its original `acd5b02d3` state.

**Why**

With the flipped-normal split, bottom-facing facets of a laid cylinder were classified as shallow. The raster tried to machine them from above, the column clearance test failed (the plunge column passes through cylinder material), and the raster emitted zero moves for those lines. The rings then ran alone and left a circumferential gap at the bottom of the cylinder. Reading the original normal sends bottom-facing facets to the rings, which cover the full circumference.

**What was considered and dropped**

- Keeping the flipped-normal split and fixing `column_clear` instead: rejected, because bottom-facing facets fundamentally cannot be machined from above in the laid frame — no column clearance fix can make them reachable without gouging.
- Keeping the `scan.rs` steep-edge changes: rejected, they produced wrong retract positions on laid bars (issue reported by user in GUI testing).

**Cost to change later**

Low. The split is a single boolean in `Sup.shallow`; flipping it back is a one-line change. The scan.rs revert is also a single file. No data format or API changes.
