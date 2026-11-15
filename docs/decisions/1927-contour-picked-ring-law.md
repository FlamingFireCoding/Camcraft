## #1927 - 2D Contour: a picked ring whose side stands in the part is refused

**Decided.** A 2D Contour on PICKED rings (the Selection source, on a
part solid) goes through the new `machining2d::contour_on_picked`. After
the per-ring depth ladder, `contour_depth::picked_law` refuses the
operation by name when a picked ring walks no level under the part top
near it, while the operation's own levels go under that top. The
message names the ring's position, the part top and the fix:
"Outside of this ring is inside the part - it rings a hole; use Inside
or Auto" (and the mirror text for Inside on a ring that bounds
material). The whole-model source (`SrcRef::Entity` / stock) keeps
`contour_on_bodies` and is not changed.

**Why.** The depth ladder judges each level against the part material
strictly above that level. At or above the part top nothing stands
there, so every ring is legal at the top plane. Side Outside on a hole
wall therefore kept only the top level (Z = part top), cut air there
with the centre one radius inside the plate's footprint, and posted
with no warning, at any Bottom. The verifier saw no gouge because the
tip only touched the top face. The ring did not gouge, but the
operation did not machine what the user picked and said nothing: a
silent wrong answer under the Prime Directive. The verifier does
convict the same ring when it is forced down the full ladder (pinned by
`the_full_depth_outside_ring_is_a_gouge_to_the_verifier`), so no
verifier gap was filed.

"Near the ring" is the highest vertex of the facets whose XY box comes
within the tool radius plus stock to leave of the ring's XY box. This
keeps a ring on a low body from being judged against a taller,
unrelated body in a multi-body pick.

**Dropped.**
- Flip the side silently to Inside: a strategy fork the user chose by
  name must not change without a word.
- Refuse in the app Selection arm by reading the path notes: fragile
  string matching across crates.
- Apply the law to whole-model sources too: their rings come from the
  part's own section, and a chamfer or fillet ring near the top could
  lose its top-only pass today. Out of scope for this defect.
- A new `ContourParams` field for "picked": it would touch every struct
  literal in the test suites for no gain over a second entry point.

**Cost to change later.** Low. The law is one function in
`contour_depth.rs` and one call in `contour_impl`. `contour.rs` was
already over the 400-line law before this PR (446 lines); this PR adds
15 lines (the entry point and the call). A split of `contour_impl` is owed separately.

When Side is Auto and Auto's fill verdict resolves to the wrong side,
the same refusal fires with the resolved side's text ("use Inside or
Auto"). That case was not seen on any fixture.
