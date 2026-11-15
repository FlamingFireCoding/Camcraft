## #1924 - Red main: fix the stale fixture of the threaded-batch test, not the reach law

**What was decided.** `threaded_batch_matches_the_serial_loop_bit_for_bit`
(`crates/app/tests/operations.rs`) now uses a 60 x 60 x 5 box stock, not
60 x 60 x 20. The stock stands to the top of the 30 x 30 x 5 cube, as the
sibling test `the_shell_switch_sends_the_pocket_to_the_offset_engine`
already does since #96. No production code changed.

**Why.** After #1895 (#1844), generation of the `APocket3D` job refused
with: "Bottom of Part Z is 20.000 mm below the material top, but its wider
shank starts 15.000 mm above the tip". The refusal is correct for that
fixture:

- the stock top was Z 20, the pocket top is Z 5 and Bottom Z is 0;
- no operation ahead of the pocket removes the stock from Z 5 to Z 20;
- the catalog BN-0600 has a 15 mm flute and an 8 mm shank, wider than the
  6 mm ball, so at Bottom Z 0 the shank is in uncut stock from Z 15 to 20.

The test proves that the threaded batch and the serial loop write the same
bytes. It does not need a deep stock. With the stock at Z 5 the depth is
5 mm, under the 15 mm reach, and the three jobs (pocket, contour at
Bottom Z 4, face at Z 4.5) generate again.

**Considered and dropped.**

- A longer-flute BN-0600 for this test (as `add_swarf_ball` does). Dropped:
  it hides the uncut stock between Z 5 and 20, which the rapids at
  clearance Z 10 also go into (the #96 note).
- A change to the reach law. Dropped: the law is correct here, and the
  task forbids to weaken it.

**Cost to change later.** One fixture value in one test.

The second failure (`docs_troubleshooting.rs`) is a path update only: #1908
moved the page to `site/content/guide/reference/troubleshooting.md`, by
design. No decision was needed.
