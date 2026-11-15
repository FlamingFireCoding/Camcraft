## #1172 - Wire EDM fan pairing and stitched profile refusals

**Decided:** Sync pairs may hold a point of either profile while the other
profile walks a stated span. The correspondence is weakly monotone in arc
length. Sync pairs sharing a lower point keep their stated order. Pairs
sharing an upper point follow their order along the lower ring. A crossed
order is refused. A zero-length span keeps its apex on the profile plane.
The existing fold, crossing, kerf and maximum-lean laws still judge the
stated surface and the offset wire.

A fan may straddle the cyclic profile seam. Once the upper endpoint has
advanced, returning to its initial point ends the current turn rather
than starting another. The pin
`an_upper_apex_fan_can_wrap_the_lower_ring_seam` covers this case.

Identical rulings remain invalid. Several anchors all at one point of
either profile are also refused: the return span would ambiguously mean
zero distance or a whole turn around that profile.

Several picked open runs on the same plane receive a named refusal for a
stitched profile. The user can pick a face or one closed edge chain.
The reader does not infer a join between independent picks. This follows
the issue's completion criterion allowing either a single profile or a
named refusal.

**Why:** A fan is an explicit cone over its stated span. Allowing a
stationary endpoint preserves that form while retaining the safety laws
that already judge ruled cuts. An explicit stitched-profile refusal is
the smallest safe change without introducing a second joining policy.

**Considered and dropped:** Removing duplicate-point checks without
checking order would admit crossed or ambiguous correspondences. Guessing
how separate runs join could connect unintended geometry. Inferring fan
spans from nearby corners would change the surface without a stated sync
pair.

**Cost to change later:** Supporting stitched runs needs a planar joining
policy with pins for gaps, branches, duplicates and disconnected rings.
Changing fan ordering needs a stated representation for the span direction
and must retain the cyclic-seam pin and existing wire laws.
