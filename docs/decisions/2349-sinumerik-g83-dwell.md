## #2349 - Sinumerik keeps the G83 dwell as P in ms

**Decided:** The Sinumerik post writes the bottom dwell of G83 (deep
peck) as `P<ms>`: whole milliseconds, before the F word, as it already
does on G82. This is the behaviour of main; the first version of PR
#2353 refused the dwell, and that refusal is removed. G73, G84 and the
G86 dwell refusal do not change. The post writes no G291: it assumes
that the control runs in ISO dialect mode.

**Why:** The maintainer verified the word on a control. His words,
2026-11-17: "Can verify the G83 P dwell word works if you run (840D i
iso mode (G291))". In the decision dialog he then chose:
- G83: "Keep P, no refusal".
- P unit: "Yes, milliseconds".
- G73 dwell (#2354): "Post P as on G83".

**Sources read:**
- Siemens, *SINUMERIK ONE / 840D sl / 828D ISO dialects*, Function
  Manual, 07/2021 (6FC5397-7BP40-3BA2), named by the maintainer. Read:
  section 6.3 (drilling cycles, ISO dialect M: overview table 6-2 and
  shell cycles CYCLE381M, CYCLE383M, CYCLE384M), the GUD list of the
  shell cycles, section 4.2.10 (G04 dwell, MD20734 bit 2), and SD 55810.
  - Table 6-2 lists P on G82, G84 and G89, and no P on G73, G83, G85 and
    G86.
  - G73 and G83 call CYCLE383M. The GUD dwell at final depth is listed
    for G82, G89, G76 and G87 only, not for G73 or G83.
  - SD 55810 sets seconds or revolutions for the G83/G87 dwell, but only
    in ISO dialect T (turning), not in ISO dialect M.
  - This disagrees with the maintainer's verification of P on G83. The
    post keeps G83 as on main (P in ms), because that is what the
    maintainer saw on the control.
  - G73: neither the control check nor the manual supports P. The post
    keeps G73 as on main. #2354 stays open for the maintainer.
- Siemens document 109783242 (support.industry.siemens.com): named by
  the maintainer, not read. The page needs JavaScript and gave no text.

**Considered and dropped:**
- Refuse a G83 dwell by name (the first version of PR #2353): the
  maintainer verified that the control reads P, so the refusal would
  refuse a legal program.
- Post P on G73 (the maintainer's dialog choice): section 6.3 of the
  manual lists no P on G73, and the control check was for G83 only. The
  coordinator rule for a disagreement keeps the disputed part as on main.
- Post the native `CYCLE83` with its dwell parameter (DTB, seconds):
  this changes each G83 that the post writes and mixes a native cycle
  into the ISO-form cycle group.
- Write a G291 line in the header: not asked for; the post keeps its
  assumption that the control is in ISO dialect mode.

**Cost to change later:** A control in Siemens mode (G290) does not read
these cycles. To support it, the post must write `CYCLE83` / `CYCLE82`
or a G291 line, with new goldens.

**Validation:** `crates/post/tests/sinumerik_g83_dwell.rs` pins
`G83 ... P500` for a 0.5 s dwell, G83 with no dwell and no P, and G82
`P500`. No golden changes.
