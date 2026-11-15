## #1927 - 2D Contour: a picked ring whose side stands in the part is refused

**Decided.** A 2D Contour on PICKED rings (the Selection source, on a
part solid) goes through the new `machining2d::contour_on_picked`. After
the per-ring depth ladder, `contour_depth::picked_law` refuses the
operation by name when a picked ring walks no level under the part top
near it, while the operation's own levels go under that top. The
message names the ring's position, the part top and the fix: "the
cutter Outside this ring stands in the part material - if the ring
bounds a hole, set Side to Inside; else check the tool containment"
(and the mirror text for Inside). The side the law reads is the
resolved one (the app turns Auto into Inside or Outside first), and a
tool containment moves the ring too, so the text names both causes and
never says "use Auto". The whole-model source (`SrcRef::Entity` / stock) keeps
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
about 20 lines (the entry point, the call and a type alias). A split of
`contour_impl` is owed separately.

**Seen during the gate.** Two tests pinned the silent top-plane pass:
- `area_open`: Side Auto on the rim of a BLIND bore resolves to Outside
  (the bore floor fills the XY silhouette that Auto's fill ratio
  reads), and a forced Containment Outside puts the ring 10 mm into the
  wall. Both walked only the mouth, in air, with 0 warnings. Both are
  now refused by name; Inside is the clean control. The Auto verdict
  itself is wrong for blind bores: filed as #1933 (P3), not fixed here.
- `tilt`: the flat control used Inside on a block's top face, the
  mirror case. It now uses Outside, the legal side for that ring.
