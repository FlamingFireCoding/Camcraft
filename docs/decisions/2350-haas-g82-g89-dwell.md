## #2350 - Haas G82 and G89 post each hole's own dwell

**Decided:** The Haas post adds the dwell to the open-cycle signature of
G82 and G89, as #1820 did for G83. A hole whose dwell differs from the
open group's closes it (G80) and re-issues the cycle with its own P.
G89 always writes P, P0 for a zero dwell; G82 is only chosen for a
positive dwell (a zero-dwell drill posts G81), so it always carries P.

**Why:** Haas documents P as the bottom dwell of G82 and G89 and, in its
G04 and G83 pages, a P that stays modal when omitted. Before this fix a
second G82/G89 hole with a different dwell was posted as a bare X Y block
and ran the first hole's dwell - wrong posted G-code (P1).

**Considered and dropped:** Writing P on every bare X Y hole of the group
(not documented as read on a hole-position-only block); refusing a
changed dwell (the control supports it, re-issuing the cycle is exact).
Shared code across posts: the defect and fix are Haas-only, other
Fanuc-family posts are handled by their own issues (#2348, #2349).

**Cost to change later:** One line of the signature and the P condition
in `assets/posts/haas.cps`; pinned by `crates/post/tests/haas_dwell_cycles.rs`.
No real Haas control was exercised.
