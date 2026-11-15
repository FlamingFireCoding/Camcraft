## #1625 - Pin cross-hole stamp digest and correct lattice wording in hole_digest.rs

**What was decided.** Pin every `CrossHole` field (`origin.x`, `origin.y`,
`origin.z`, `dir.x`, `dir.y`, `dir.z`, `radius`, `entry_d`, `floor_d`,
and `through`) in `crates/app/src/operation/hole_digest.rs` through both
`cross()` and `OpParams::Drill::stamp_with`. Each float field is moved by
one lattice step (`1e-6` mm), an off-lattice `floor_d` (`-(0.190 in -
0.040 in)`) is checked against its `-3.81` mm snapped form, sub-half-step
noise (`2e-7` mm) is pinned to keep the stamp unchanged on snapped fields,
and sub-lattice changes on `dir` are pinned to move the stamp (matching
`CrossHoleDto`'s verbatim `dir`). The comments in `hole_digest.rs` are
updated from `1 um` / `half a micron` to `1 nm (1e-6 mm)`.

**Why.** `hole_digest::cross` already digested all ten `CrossHole`
fields, so the gap was missing test coverage rather than missing hash
calls. Pinning both `cross()` and `OpParams::Drill::stamp_with` guards
both the digest helper and its Drill-arm caller in `operation/params.rs`.
`docs/decisions/1560-hole-digest-lattice.md` was already gathered into
`RECOMMENDED.md` during the v0.20.0 cut, and `RECOMMENDED.md` is cut-only
and must not be edited on an issue branch.

**Considered and dropped.**
- Editing the gathered `#1560` entry in `RECOMMENDED.md`: skipped because
  issue branches never modify `RECOMMENDED.md`.
- Editing `crates/cl/src/cl/moves.rs` (`lattice` doc comment) or
  `crates/app/tests/bore_{core,entry}.rs`: outside #1625's stated scope,
  and touching `crates/cl` would widen the quick gate to batch.
- Adding a full project save/open test for `CrossHole`: unneeded here
  since `CrossHoleDto::from_hole` uses the same `cl::lattice` function
  and `CrossHole` unit pins prove field coverage directly.

**Cost to change later.** Negligible; the changes are confined to comments
and `#[cfg(test)]` functions in `crates/app/src/operation/hole_digest.rs`.
