## #2360 - G82/G89 dwell in the cycle signature (Fanuc, Mazak, Doosan, grblHAL, Tormach)

**What was decided.**
- The dwell is part of the cycle signature for G82 and G89 on the Fanuc,
  Mazak and Doosan posts, and for G82, G86 and G89 on the grblHAL and
  Tormach posts (they write a G86 P). A changed dwell writes G80 and a new
  cycle block with its own P. This is the pattern of PR #2355 (Haas).
- A dwell of 0 writes no P word and a cycle that does not read P:
  - a drill or spot hole goes to G81 (as before);
  - a back-bore goes to G85 (feed in, feed out), not to G89 without P.

**Why.** Fanuc's published G82/G89 formats and the LinuxCNC G-code
reference give P as the dwell, and the LinuxCNC reference keeps numbers
sticky until they are programmed again. A bare X Y hole runs the modal P.
For a zero dwell, no public source that we read confirms that `P0` is
read as "no dwell" on Fanuc, Mazak or Doosan. The LinuxCNC reference
lists "a P number is required" as a canned-cycle error, and the LinuxCNC
interpreter (`interp_cycles.cc`, `convert_cycle_xy`) refuses a G82, G86
or G89 block without P when the motion mode was not already that cycle
(after G80). A G89 with no P stops a LinuxCNC-family control; on the
Fanuc family it would rely on G80 clearing the modal P, which we did not
verify. G85 is the same motion as G89 with a zero dwell
on all of these controls, and it has no P word. So the zero dwell needs
no unverified word.

**Dropped.**
- `P0` on G89, as Haas writes it (#2350): Haas documents P0; the other
  five controls were not verified for it.
- The dwell in the signature of every cycle: G73, G83 (dwell refused),
  G84 and G85 read no P, so a dwell change there must not split a group.

- A zero-dwell G86 on grblHAL and Tormach: it still posts with no P,
  which the LinuxCNC interpreter refuses after G80. That is #2362, not
  this fix.

**Cost to change later.** One line per post. The pinning tests in
`crates/post/tests/dwell_cycles_modal_p.rs` state the expected words.
