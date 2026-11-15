## #1053 - Cross drill judged on reduced-fidelity models

**What was decided.** Remove cross drill from `fidelity_law::excess_aware`'s
refused list and propagate `material.excess` through every construction
and verification law in the cross-drill chain:

- `fidelity_law::band_growth(excess)` (`(GOUGE_TOL - tightened(GOUGE_TOL, excess)).max(0.0)`)
  gives the outward standoff growth while `GOUGE_TOL` stays positive and
  returns `0.0` once past it, where the program gate refuses by name.
- `cross_stock::standoff` grows measured `r`, `raw`, and `z_max` by `band_growth(excess)`.
- `cross_entry::tube_max`, `entry_grown`, and `tube_reach` offset each
  facet's sagitta slab along its normal by `s + grow` (matching a convex
  Minkowski allowance for chord uncertainty on both flat faces and curved bar
  sections).
- `drill_reach::cross_reach` adds `part_grow` (`st.grow` when `st.raw` is
  `None`, `0.0` when `to_raw_mouths` already shifted mouths via `entry_grown`).
- `drill_station::material` and `drill_reach::cross_holder_excess` grow
  `raw`, station slabs, and swing Z-band / radius checks by `band_growth(excess)`.
- `cross_part::part_clear` and `neighbours_clear` (both in `cross_spindle`
  and `kept_rejudge`) tighten `SolidLaw::allow` via `tightened(GOUGE_TOL, excess)`.
- `camcraft_sim::verify::rapids_radial::radial_rapids` grows `top` by
  `cx.band_growth()` and tightens `disc` using `cx.tightened(...)`.

**Why.** Every construction law that proves a cross drill clear against
measured part/stock geometry previously read coarse facets directly with no
tolerance slack. Using `band_growth(excess)` ensures coarse-model standoffs
grow smoothly when `0 < excess < GOUGE_TOL`, do not blow up on pre-v59
`UNKNOWN_MM` or past-bound models (since the gate refuses them), and keep
full-fidelity output bit-for-bit unchanged (`0.0`). Expanding facet slabs along
normals in `cross_entry::tube_max` rather than adding `grow` only at the end
catches both outward entry shifts and transverse wall/flange clearances on a
station move without double-counting when `to_raw_mouths` shifts hole depths.

**Considered and dropped.**
- Using raw `excess` instead of `band_growth(excess)`: unsafe when `excess` is
  `UNKNOWN_MM` (`1e12`) or past the floor bound because station standoffs would
  expand wildly even though verification is supposed to fall back to
  untightened values behind the gate refusal.
- Adding `grow` in both `to_raw_mouths` and `cross_reach`: would double-count
  `excess` on raw-stock cross holes.

**Cost to change later.** Low: `band_growth` lives in one helper in
`fidelity_law.rs`, and `excess` is explicitly passed through the cross-drill
entry points.
