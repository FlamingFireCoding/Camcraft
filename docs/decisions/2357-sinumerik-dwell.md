## #2357 - Refuse unsupported Sinumerik tapping and reaming dwells

**Chosen:** Refuse positive G84 (tap) and G85 (ream) dwells by name before
positioning or writing the cycle. The issue explicitly permits a G84 refusal.
Siemens' [ISO dialects Function Manual, 07/2021, section 6.3, table 6-2](https://cache.industry.siemens.com/dl/files/254/109801254/att_1077768/v1/ONE_840Dsl_828D_iso_dialects_fct_man_0721_en-US.pdf)
lists a P address for G84 and none for G85. The G84 diagnostic describes this
post's limitation; it does not claim the controller lacks tapping dwell.

**Why:** The post silently dropped TAP dwell and emitted an unsupported P
address for REAM dwell. The motion generator currently gives Tap and Ream
zero dwell, but the public CycleRec serializer and parser preserve positive
dwells. The regression fixtures establish that premise before posting them.
Test the requested seconds before millisecond rounding so a hand-authored
sub-millisecond request cannot disappear. Track a refused hole with the
existing cycle coordinates and inCycle flag to suppress its expanded G1
cuts. This needs no new state or helper.

**Dropped:** Silently omitting dwell, inventing a G85 P address, changing
generator behavior, and adding G84 P output without verifying its documented
units and controller semantics. A P address in a cycle table does not establish
that every cycle uses the same encoding.

**Cost to change later:** A documented G84 implementation would replace its
refusal and update its pins. G85 remains a named refusal unless a source
supports a different representation.

**Validation:** Both refusal pins fail with the old upstream post and pass
with the fix. Removing refused-hole tracking makes both pins fail on leaked
G1 cuts; restoring it makes them pass. Zero-dwell controls pass. The quick
gate passed all 46 steps, and camcraft-post clippy passed for all targets
with warnings denied. No golden files changed. No physical controller was
exercised.
